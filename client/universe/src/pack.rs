//! Incremental, content-addressed PRIVATE development packs. No production gate bypass.
use crate::{
    Catalog, CatalogId, Error, invalid,
    partitions::{self, CellKey},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Write,
    path::{Path, PathBuf},
};

pub const LEGACY_MANIFEST_BYTES: usize = 16 * 1024 * 1024;
/// V2 admits the explicitly bounded nearby catalog, not a full Gaia index.
pub const MAX_MANIFEST_BYTES: usize = 64 * 1024 * 1024;
mod observed;
pub mod pipeline;
pub const MAX_BLOB_BYTES: usize = partitions::MAX_PACK_BYTES;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Blob {
    pub sha256: String,
    pub bytes: u64,
    /// Present only for a version-2 lossless zlib blob. Digest/bytes bind the
    /// stored stream; this separately bounds its exact decoded byte count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zlib_raw_bytes: Option<u64>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cell {
    pub key: CellKey,
    pub catalog: Blob,
    pub record_ids: Vec<CatalogId>,
    pub sidecars: BTreeMap<String, Blob>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub format: String,
    pub version: u32,
    pub revision: String,
    pub production_eligible: bool,
    pub source_manifest: Blob,
    pub cells: Vec<Cell>,
    pub migrations: BTreeMap<CatalogId, CatalogId>,
}
pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn io(e: std::io::Error) -> Error {
    Error::Encoding(format!("pack IO: {e}"))
}
pub fn blob_path(root: &Path, blob: &Blob) -> Result<PathBuf, Error> {
    if blob.sha256.len() != 64
        || !blob
            .sha256
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        || blob.bytes == 0
        || blob.bytes > MAX_BLOB_BYTES as u64
        || blob
            .zlib_raw_bytes
            .is_some_and(|n| n == 0 || n > MAX_BLOB_BYTES as u64)
    {
        return Err(invalid("pack blob digest/size"));
    }
    Ok(root.join("blobs").join(&blob.sha256))
}
pub fn read_blob(root: &Path, blob: &Blob) -> Result<Vec<u8>, Error> {
    let path = blob_path(root, blob)?;
    let file = fs::File::open(path).map_err(io)?;
    use std::io::Read;
    let mut bytes = Vec::new();
    file.take(blob.bytes + 1)
        .read_to_end(&mut bytes)
        .map_err(io)?;
    if bytes.len() as u64 != blob.bytes || digest(&bytes) != blob.sha256 {
        return Err(invalid("pack blob size/SHA256 mismatch"));
    }
    if let Some(raw_bytes) = blob.zlib_raw_bytes {
        // An extra output byte detects a dishonest size without unbounded
        // inflation. Require a complete single stream, including its checksum.
        let mut raw = vec![0; raw_bytes as usize + 1];
        let mut decoder = flate2::Decompress::new(true);
        let status = decoder
            .decompress(&bytes, &mut raw, flate2::FlushDecompress::Finish)
            .map_err(|e| Error::Encoding(format!("pack zlib: {e}")))?;
        if status != flate2::Status::StreamEnd
            || decoder.total_in() != blob.bytes
            || decoder.total_out() != raw_bytes
        {
            return Err(invalid("pack zlib stream/decoded size mismatch"));
        }
        raw.truncate(raw_bytes as usize);
        return Ok(raw);
    }
    Ok(bytes)
}
impl Manifest {
    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() > MAX_MANIFEST_BYTES {
            return Err(invalid("pack manifest size"));
        }
        let m: Self = serde_json::from_slice(bytes).map_err(|e| Error::Encoding(e.to_string()))?;
        if m.version == 1 && bytes.len() > LEGACY_MANIFEST_BYTES {
            return Err(invalid("version-1 pack manifest size"));
        }
        m.validate()?;
        Ok(m)
    }
    pub fn validate(&self) -> Result<(), Error> {
        if self.format != "salimon.universe.private-pack"
            || !matches!(self.version, 1 | 2)
            || self.production_eligible
            || self.revision.trim().is_empty()
            || self.cells.is_empty()
            || self.cells.len() > partitions::MAX_CELLS
            || self.cells.windows(2).any(|c| c[0].key >= c[1].key)
        {
            return Err(invalid("private pack format/version/order/scope"));
        }
        blob_path(Path::new("."), &self.source_manifest)?;
        if self.version == 1
            && std::iter::once(&self.source_manifest)
                .chain(
                    self.cells
                        .iter()
                        .flat_map(|c| std::iter::once(&c.catalog).chain(c.sidecars.values())),
                )
                .any(|b| b.zlib_raw_bytes.is_some())
        {
            return Err(invalid("version-1 pack cannot contain compressed blobs"));
        }
        let mut ids = BTreeSet::new();
        for c in &self.cells {
            blob_path(Path::new("."), &c.catalog)?;
            if c.record_ids.windows(2).any(|i| i[0] >= i[1])
                || c.record_ids.iter().any(|id| !ids.insert(id))
            {
                return Err(invalid("pack duplicate/unsorted identity"));
            }
            for (name, blob) in &c.sidecars {
                if name.is_empty()
                    || name.len() > 80
                    || !name
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
                {
                    return Err(invalid("pack sidecar name"));
                }
                blob_path(Path::new("."), blob)?;
            }
        }
        if self
            .migrations
            .iter()
            .any(|(old, new)| old == new || ids.contains(old) || !ids.contains(new))
        {
            return Err(invalid("pack identity migration"));
        }
        Ok(())
    }
    pub fn encode(&self) -> Result<Vec<u8>, Error> {
        self.validate()?;
        let bytes = serde_json::to_vec(self).map_err(|e| Error::Encoding(e.to_string()))?;
        let limit = if self.version == 1 {
            LEGACY_MANIFEST_BYTES
        } else {
            MAX_MANIFEST_BYTES
        };
        if bytes.len() > limit {
            return Err(invalid("pack manifest size"));
        }
        Ok(bytes)
    }
}
pub struct Builder {
    root: PathBuf,
    manifest: Manifest,
    sources: BTreeMap<CatalogId, crate::Dataset>,
    ids: BTreeSet<CatalogId>,
}
impl Builder {
    pub fn new(root: &Path, revision: String, source_manifest: &[u8]) -> Result<Self, Error> {
        // Validate metadata without claiming human review, full data or source permission.
        crate::sources::SourceManifest::decode(source_manifest, false)?;
        fs::create_dir_all(root.join("blobs")).map_err(io)?;
        let mut b = Self {
            root: root.into(),
            sources: BTreeMap::new(),
            ids: BTreeSet::new(),
            manifest: Manifest {
                format: "salimon.universe.private-pack".into(),
                version: 2,
                revision,
                production_eligible: false,
                source_manifest: Blob {
                    sha256: String::new(),
                    bytes: 0,
                    zlib_raw_bytes: None,
                },
                cells: vec![],
                migrations: BTreeMap::new(),
            },
        };
        b.manifest.source_manifest = b.store(source_manifest)?;
        Ok(b)
    }
    fn store(&self, bytes: &[u8]) -> Result<Blob, Error> {
        self.store_encoded(bytes, None)
    }
    fn store_sidecar(&self, raw: &[u8]) -> Result<Blob, Error> {
        if raw.is_empty() || raw.len() > MAX_BLOB_BYTES {
            return Err(invalid("pack sidecar decoded size"));
        }
        let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::new(6));
        encoder.write_all(raw).map_err(io)?;
        let compressed = encoder.finish().map_err(io)?;
        if compressed.len() < raw.len() {
            self.store_encoded(&compressed, Some(raw.len() as u64))
        } else {
            self.store(raw)
        }
    }
    fn store_encoded(&self, bytes: &[u8], zlib_raw_bytes: Option<u64>) -> Result<Blob, Error> {
        let blob = Blob {
            sha256: digest(bytes),
            bytes: bytes.len() as u64,
            zlib_raw_bytes,
        };
        let path = blob_path(&self.root, &blob)?;
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut f) => {
                if let Err(e) = f.write_all(bytes).and_then(|_| f.sync_all()) {
                    let _ = fs::remove_file(path);
                    return Err(io(e));
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                read_blob(&self.root, &blob)?;
            }
            Err(e) => return Err(io(e)),
        }
        Ok(blob)
    }
    /// One dependency-closed cell at a time. Sidecars preserve exact typed producer bytes.
    pub fn add_cell(
        &mut self,
        key: CellKey,
        catalog: &Catalog,
        sidecars: &BTreeMap<String, Vec<u8>>,
    ) -> Result<(), Error> {
        if catalog.revision() != self.manifest.revision {
            return Err(invalid("pack revision mismatch"));
        }
        if self.manifest.cells.len() >= partitions::MAX_CELLS {
            return Err(invalid("pack cell count"));
        }
        let insertion = self
            .manifest
            .cells
            .binary_search_by_key(&key, |c| c.key)
            .map_or_else(Ok, |_| Err(invalid("pack duplicate cell")))?;
        let mut sources = self.sources.clone();
        for s in catalog.datasets() {
            if sources
                .insert(s.id.clone(), s.clone())
                .is_some_and(|old| old != *s)
            {
                return Err(invalid("pack conflicting source"));
            }
        }
        if catalog
            .records()
            .iter()
            .any(|r| self.ids.contains(r.id()) || sources.contains_key(r.id()))
            || sources.keys().any(|id| self.ids.contains(id))
        {
            return Err(invalid("pack duplicate record/source identity"));
        }
        let payload = partitions::encode(vec![(key, catalog.clone())])?;
        let mut blobs = BTreeMap::new();
        for (name, bytes) in sidecars {
            blobs.insert(name.clone(), self.store_sidecar(bytes)?);
        }
        let cell = Cell {
            key,
            catalog: self.store(&payload)?,
            record_ids: catalog.records().iter().map(|r| r.id().clone()).collect(),
            sidecars: blobs,
        };
        // Validate only this new dependency-closed cell. Cloning and revalidating
        // the entire growing catalog on each batch made nearby imports quadratic.
        // The publication is fully validated once again by finish/encode.
        for name in cell.sidecars.keys() {
            if name.is_empty()
                || name.len() > 80
                || !name
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
            {
                return Err(invalid("pack sidecar name"));
            }
        }
        self.ids.extend(cell.record_ids.iter().cloned());
        self.manifest.cells.insert(insertion, cell);
        self.sources = sources;
        Ok(())
    }
    /// Reject silent deletion/rekeying against the previous publication.
    pub fn finish(
        mut self,
        previous: Option<&Manifest>,
        migrations: BTreeMap<CatalogId, CatalogId>,
    ) -> Result<Manifest, Error> {
        self.manifest.migrations = migrations;
        if let Some(old) = previous {
            old.validate()?;
            let ids: BTreeSet<_> = self
                .manifest
                .cells
                .iter()
                .flat_map(|c| &c.record_ids)
                .collect();
            let old_ids: BTreeSet<_> = old.cells.iter().flat_map(|c| &c.record_ids).collect();
            if old_ids
                .iter()
                .any(|id| !ids.contains(*id) && !self.manifest.migrations.contains_key(*id))
                || self
                    .manifest
                    .migrations
                    .keys()
                    .any(|id| !old_ids.contains(id))
            {
                return Err(invalid("pack removed identity requires explicit migration"));
            }
        } else if !self.manifest.migrations.is_empty() {
            return Err(invalid("pack migration requires previous manifest"));
        }
        let bytes = self.manifest.encode()?;
        let temp = self.root.join("manifest.json.pending");
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(io)?;
        let result = (|| {
            f.write_all(&bytes).map_err(io)?;
            f.sync_all().map_err(io)?;
            fs::rename(&temp, self.root.join("manifest.json")).map_err(io)
        })();
        if result.is_err() {
            let _ = fs::remove_file(temp);
        }
        result?;
        Ok(self.manifest)
    }
}
#[cfg(test)]
mod tests;
