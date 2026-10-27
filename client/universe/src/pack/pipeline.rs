//! Standalone, local-input build orchestration. Downloads remain acquisition
//! tooling; every selected byte is frozen before this offline transaction.
use super::*;
use serde::Deserialize;
use std::io::Read;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub version: u32,
    pub revision: String,
    pub source_manifest: String,
    pub cells: Vec<CellInput>,
    #[serde(default)]
    pub migrations: BTreeMap<CatalogId, CatalogId>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CellInput {
    pub key: CellKey,
    /// A U01 catalog, or a producer bundle with a `catalog` field. Every other
    /// bundle field is retained as an exact JSON sidecar (observations, source
    /// metadata, inference evidence, full generation request/definitions).
    pub input: String,
    #[serde(default)]
    pub sidecars: BTreeMap<String, String>,
}
fn local(root: &Path, path: &str) -> Result<PathBuf, Error> {
    let relative = Path::new(path);
    if relative.is_absolute()
        || relative
            .components()
            .any(|c| !matches!(c, std::path::Component::Normal(_)))
    {
        return Err(invalid("pack input path must be root-relative"));
    }
    let path = root.join(relative).canonicalize().map_err(io)?;
    if !path.starts_with(root) {
        return Err(invalid("pack input symlink escapes root"));
    }
    Ok(path)
}
fn bounded(path: &Path, limit: usize) -> Result<Vec<u8>, Error> {
    let mut out = Vec::new();
    fs::File::open(path)
        .map_err(io)?
        .take(limit as u64 + 1)
        .read_to_end(&mut out)
        .map_err(io)?;
    if out.len() > limit {
        return Err(invalid("pack input exceeds byte limit"));
    }
    Ok(out)
}
/// Validate exact original source snapshots with streaming SHA256. Private
/// observed snapshots are admitted without fabricating a release review record.
fn verify_sources(path: &Path, bytes: &[u8]) -> Result<(), Error> {
    let manifest = crate::sources::SourceManifest::decode(bytes, false)?;
    let root = path
        .parent()
        .ok_or_else(|| invalid("source manifest parent"))?;
    for source in &manifest.sources {
        for artifact in &source.artifacts {
            let path = local(root, &artifact.path)?;
            let mut file = fs::File::open(path).map_err(io)?;
            let mut hasher = Sha256::new();
            let mut size = 0_u64;
            let mut buffer = [0_u8; 65536];
            loop {
                let n = file.read(&mut buffer).map_err(io)?;
                if n == 0 {
                    break;
                }
                size = size
                    .checked_add(n as u64)
                    .ok_or_else(|| invalid("source byte count overflow"))?;
                if size > artifact.bytes {
                    return Err(invalid("source artifact exceeds declared bytes"));
                }
                hasher.update(&buffer[..n]);
            }
            if size != artifact.bytes || format!("{:x}", hasher.finalize()) != artifact.sha256 {
                return Err(invalid(format!(
                    "source artifact size/hash: {}",
                    artifact.path
                )));
            }
        }
    }
    Ok(())
}
pub fn build(request_file: &Path, output: &Path) -> Result<Manifest, Error> {
    let request_file = request_file.canonicalize().map_err(io)?;
    let root = request_file
        .parent()
        .ok_or_else(|| invalid("pack request parent"))?;
    let request: Request = serde_json::from_slice(&bounded(&request_file, MAX_MANIFEST_BYTES)?)
        .map_err(|e| Error::Encoding(e.to_string()))?;
    if request.version != 1
        || request.cells.is_empty()
        || request.cells.len() > partitions::MAX_CELLS
    {
        return Err(invalid("pack request version/cell bound"));
    }
    let source_path = local(root, &request.source_manifest)?;
    let source_bytes = bounded(&source_path, 1024 * 1024)?;
    verify_sources(&source_path, &source_bytes)?;
    let previous = match fs::File::open(output.join("manifest.json")) {
        Ok(mut f) => {
            let mut b = Vec::new();
            std::io::Read::by_ref(&mut f)
                .take(MAX_MANIFEST_BYTES as u64 + 1)
                .read_to_end(&mut b)
                .map_err(io)?;
            Some(Manifest::decode(&b)?)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(io(e)),
    };
    let mut builder = Builder::new(output, request.revision.clone(), &source_bytes)?;
    for cell in request.cells {
        let bytes = bounded(
            &local(root, &cell.input)?,
            crate::synthesis::MAX_PUBLICATION_BYTES,
        )?;
        let mut sidecars = BTreeMap::new();
        let value: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|e| Error::Encoding(e.to_string()))?;
        let catalog = if let Some(catalog) = value.get("catalog") {
            let fields = value
                .as_object()
                .ok_or_else(|| invalid("producer bundle object"))?;
            for (key, value) in fields {
                if key != "catalog" {
                    sidecars.insert(
                        key.clone(),
                        serde_json::to_vec(value).map_err(|e| Error::Encoding(e.to_string()))?,
                    );
                }
            }
            Catalog::decode(
                &serde_json::to_vec(catalog).map_err(|e| Error::Encoding(e.to_string()))?,
            )?
        } else {
            Catalog::decode(&bytes)?
        };
        for (name, path) in cell.sidecars {
            if sidecars
                .insert(name, bounded(&local(root, &path)?, MAX_BLOB_BYTES)?)
                .is_some()
            {
                return Err(invalid("duplicate producer/explicit sidecar"));
            }
        }
        // The request explicitly names the combined publication revision;
        // source IDs, physical values and definition versions are unchanged.
        let catalog = Catalog::new(
            request.revision.clone(),
            catalog.datasets().to_vec(),
            catalog.records().to_vec(),
        )?;
        builder.add_cell(cell.key, &catalog, &sidecars)?;
    }
    builder.finish(previous.as_ref(), request.migrations)
}
