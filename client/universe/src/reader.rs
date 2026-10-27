//! Read-only private-pack queries with a bounded encoded-cell LRU.
use crate::{
    Catalog, CatalogId, Error, invalid,
    pack::{self, Manifest},
    partitions::{self, CellKey},
};
use std::{
    collections::{BTreeMap, VecDeque},
    fs,
    io::{Cursor, Read},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, mpsc},
    thread,
};

pub struct Reader {
    root: PathBuf,
    manifest: Manifest,
    ids: BTreeMap<CatalogId, usize>,
    cache: BTreeMap<CellKey, Vec<u8>>,
    lru: VecDeque<CellKey>,
    budget: usize,
    resident: usize,
    last_prefetch_error: Option<String>,
    stats: Stats,
}
#[derive(Clone, Copy, Debug, Default, serde::Serialize)]
pub struct Stats {
    pub catalog_blob_reads: u64,
    pub catalog_blob_bytes: u64,
    pub cache_hits: u64,
}
impl Reader {
    /// Budget applies to cached encoded catalog bytes (Vec capacities). Returned
    /// owned Catalogs, bounded manifest/index and one decode scratch are separate.
    pub fn open(root: &Path, budget: usize) -> Result<Self, Error> {
        if budget == 0 || budget > pack::MAX_BLOB_BYTES {
            return Err(invalid("reader cache budget"));
        }
        let file = fs::File::open(root.join("manifest.json"))
            .map_err(|e| Error::Encoding(e.to_string()))?;
        let mut bytes = Vec::new();
        file.take(pack::MAX_MANIFEST_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| Error::Encoding(e.to_string()))?;
        let manifest = Manifest::decode(&bytes)?;
        let sources = pack::read_blob(root, &manifest.source_manifest)?;
        crate::sources::SourceManifest::decode(&sources, false)?;
        let ids = manifest
            .cells
            .iter()
            .enumerate()
            .flat_map(|(i, c)| c.record_ids.iter().map(move |id| (id.clone(), i)))
            .collect();
        Ok(Self {
            root: root.into(),
            manifest,
            ids,
            cache: BTreeMap::new(),
            lru: VecDeque::new(),
            budget,
            resident: 0,
            last_prefetch_error: None,
            stats: Stats::default(),
        })
    }
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }
    pub fn resident_bytes(&self) -> usize {
        self.resident
    }
    pub fn stats(&self) -> Stats {
        self.stats
    }
    pub fn last_prefetch_error(&self) -> Option<&str> {
        self.last_prefetch_error.as_deref()
    }
    pub fn region(&self, min: [i64; 3], max: [i64; 3]) -> Result<Vec<CellKey>, Error> {
        if (0..3).any(|i| min[i] > max[i]) {
            return Err(invalid("reader reversed region"));
        }
        Ok(self
            .manifest
            .cells
            .iter()
            .filter(|c| (0..3).all(|i| c.key.galactic[i] >= min[i] && c.key.galactic[i] <= max[i]))
            .map(|c| c.key)
            .collect())
    }
    pub fn record(&mut self, id: &CatalogId) -> Result<Option<crate::Record>, Error> {
        let Some(&i) = self.ids.get(id) else {
            return Ok(None);
        };
        let catalog = self.cell(self.manifest.cells[i].key)?;
        Ok(catalog.get(id).cloned())
    }
    /// Systems have canonical record IDs and use the same exact-ID path; no
    /// display-name search, catalog-to-player revelation or runtime mutation.
    pub fn cell(&mut self, key: CellKey) -> Result<Catalog, Error> {
        if let Some(raw) = self.cache.get(&key) {
            let catalog = Catalog::decode(raw)?;
            self.stats.cache_hits = self.stats.cache_hits.saturating_add(1);
            self.touch(key);
            return Ok(catalog);
        }
        let i = self
            .manifest
            .cells
            .binary_search_by_key(&key, |c| c.key)
            .map_err(|_| invalid("unknown pack cell"))?;
        let entry = &self.manifest.cells[i];
        let bytes = pack::read_blob(&self.root, &entry.catalog)?;
        self.stats.catalog_blob_reads = self.stats.catalog_blob_reads.saturating_add(1);
        self.stats.catalog_blob_bytes = self
            .stats
            .catalog_blob_bytes
            .saturating_add(bytes.len() as u64);
        let mut reader = partitions::Reader::open(Cursor::new(bytes))?;
        if reader.cells().len() != 1
            || reader.cells()[0].key != key
            || reader.revision() != self.manifest.revision
            || reader.cells()[0].record_ids != entry.record_ids
        {
            return Err(invalid("pack manifest/cell identity mismatch"));
        }
        let catalog = reader.read_cell(key)?;
        let raw = catalog.encode()?;
        if raw.capacity() <= self.budget {
            while self.resident + raw.capacity() > self.budget {
                let old = self
                    .lru
                    .pop_front()
                    .ok_or_else(|| invalid("reader cache accounting"))?;
                if let Some(bytes) = self.cache.remove(&old) {
                    self.resident -= bytes.capacity();
                }
            }
            self.resident += raw.capacity();
            self.cache.insert(key, raw);
            self.touch(key);
        }
        Ok(catalog)
    }
    pub fn sidecar(&self, key: CellKey, name: &str) -> Result<Vec<u8>, Error> {
        let cell = self
            .manifest
            .cells
            .iter()
            .find(|c| c.key == key)
            .ok_or_else(|| invalid("unknown pack cell"))?;
        let blob = cell
            .sidecars
            .get(name)
            .ok_or_else(|| invalid("unknown pack sidecar"))?;
        pack::read_blob(&self.root, blob)
    }
    fn touch(&mut self, key: CellKey) {
        self.lru.retain(|k| *k != key);
        self.lru.push_back(key);
    }
}
/// Bounded queue; one background read/decode at a time, shared cache under a
/// mutex. Saturation returns false. Drop drains at most eight queued requests.
pub struct Prefetch {
    sender: Option<mpsc::SyncSender<CellKey>>,
    worker: Option<thread::JoinHandle<()>>,
}
impl Prefetch {
    pub fn start(reader: Arc<Mutex<Reader>>) -> Self {
        let (sender, receiver) = mpsc::sync_channel(8);
        let worker = thread::spawn(move || {
            while let Ok(key) = receiver.recv() {
                let Ok(mut reader) = reader.lock() else { break };
                if let Err(e) = reader.cell(key) {
                    reader.last_prefetch_error = Some(e.to_string());
                }
            }
        });
        Self {
            sender: Some(sender),
            worker: Some(worker),
        }
    }
    pub fn request(&self, key: CellKey) -> bool {
        self.sender
            .as_ref()
            .is_some_and(|s| s.try_send(key).is_ok())
    }
}
impl Drop for Prefetch {
    fn drop(&mut self) {
        self.sender.take();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
#[cfg(test)]
mod tests;
