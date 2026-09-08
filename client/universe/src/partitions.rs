//! U04 prototype: independently compressed, dependency-closed catalog cells.
//! CRC32 detects accidental corruption; it is not a trusted publication digest.
use crate::{Catalog, CatalogId, Dataset, Error, MAX_SCHEMA_JSON_BYTES, invalid};
use flate2::{Compression, bufread::ZlibDecoder, write::ZlibEncoder};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{Read, Seek, SeekFrom, Write},
};

const MAGIC: &[u8; 8] = b"SALUCELL";
pub const VERSION: u32 = 1;
const HEADER_BYTES: usize = 32;
pub const MAX_INDEX_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_CELLS: usize = 4096;
pub const MAX_PACK_BYTES: usize = 256 * 1024 * 1024;
/// Galactic 2^40-meter cell, plus deterministic subdivision for dense cells.
/// Placement is supplied by a science-aware builder, never inferred from names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CellKey {
    pub galactic: [i64; 3],
    pub shard: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CellIndex {
    pub key: CellKey,
    /// Relative to the first payload byte (after header and index).
    pub offset: u64,
    pub compressed_bytes: u32,
    pub raw_bytes: u32,
    pub crc32: u32,
    pub record_ids: Vec<CatalogId>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Index {
    revision: String,
    sources: Vec<Dataset>,
    cells: Vec<CellIndex>,
}
/// Deterministic prototype builder. Each cell must be a valid dependency-closed
/// U01 catalog with a shared revision, globally unique records and exact sources.
/// Full streaming construction and cross-cell references belong to U12/U13.
pub fn encode(mut cells: Vec<(CellKey, Catalog)>) -> Result<Vec<u8>, Error> {
    if cells.is_empty() || cells.len() > MAX_CELLS {
        return Err(invalid("partition cell count"));
    }
    cells.sort_by_key(|(key, _)| *key);
    let revision = cells[0].1.revision().to_owned();
    let mut sources = BTreeMap::new();
    let mut ids = BTreeSet::new();
    let mut entries = Vec::new();
    let mut payload = Vec::new();
    for (key, catalog) in cells {
        if catalog.revision() != revision
            || entries.last().is_some_and(|e: &CellIndex| e.key == key)
        {
            return Err(invalid("partition revision/duplicate cell"));
        }
        for source in catalog.datasets() {
            if ids.contains(&source.id) || sources.get(&source.id).is_some_and(|old| old != source)
            {
                return Err(invalid("partition conflicting source"));
            }
            sources.insert(source.id.clone(), source.clone());
        }
        for r in catalog.records() {
            if sources.contains_key(r.id()) || !ids.insert(r.id().clone()) {
                return Err(invalid("partition duplicate record/source identity"));
            }
        }
        let raw = catalog.encode()?;
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(6));
        encoder.write_all(&raw).map_err(io_error)?;
        let compressed = encoder.finish().map_err(io_error)?;
        if payload.len() + compressed.len() > MAX_PACK_BYTES - HEADER_BYTES {
            return Err(invalid("partition prototype pack limit"));
        }
        entries.push(CellIndex {
            key,
            offset: payload.len() as u64,
            compressed_bytes: compressed.len() as u32,
            raw_bytes: raw.len() as u32,
            crc32: crc32fast::hash(&compressed),
            record_ids: catalog.records().iter().map(|r| r.id().clone()).collect(),
        });
        payload.extend(compressed);
    }
    let index = serde_json::to_vec(&Index {
        revision,
        sources: sources.into_values().collect(),
        cells: entries,
    })
    .map_err(|e| Error::Encoding(e.to_string()))?;
    let total = HEADER_BYTES + index.len() + payload.len();
    if index.len() > MAX_INDEX_BYTES || total > MAX_PACK_BYTES {
        return Err(invalid("partition prototype index/pack limit"));
    }
    let mut out = Vec::with_capacity(total);
    out.extend(MAGIC);
    out.extend(VERSION.to_le_bytes());
    out.extend((index.len() as u32).to_le_bytes());
    out.extend((total as u64).to_le_bytes());
    out.extend(crc32fast::hash(&index).to_le_bytes());
    out.extend(0_u32.to_le_bytes());
    out.extend(index);
    out.extend(payload);
    Ok(out)
}
fn io_error(e: std::io::Error) -> Error {
    Error::Encoding(format!("partition IO: {e}"))
}
fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().expect("fixed header field"))
}
/// Validates the bounded index on open. Only requested cells are read/inflated.
/// A cell is not trusted until its CRC, stream, U01 schema and index agree.
pub struct Reader<R> {
    input: R,
    index: Index,
    payload_start: u64,
    record_cells: BTreeMap<CatalogId, usize>,
}
impl<R: Read + Seek> Reader<R> {
    pub fn open(mut input: R) -> Result<Self, Error> {
        input.seek(SeekFrom::Start(0)).map_err(io_error)?;
        let mut header = [0; HEADER_BYTES];
        input.read_exact(&mut header).map_err(io_error)?;
        if &header[..8] != MAGIC || u32_at(&header, 8) != VERSION || u32_at(&header, 28) != 0 {
            return Err(invalid("partition magic/version/reserved"));
        }
        let index_len = u32_at(&header, 12) as usize;
        let declared_total = u64::from_le_bytes(header[16..24].try_into().expect("fixed header"));
        let actual_total = input.seek(SeekFrom::End(0)).map_err(io_error)?;
        if index_len == 0
            || index_len > MAX_INDEX_BYTES
            || actual_total != declared_total
            || actual_total > MAX_PACK_BYTES as u64
            || actual_total < (HEADER_BYTES + index_len) as u64
        {
            return Err(invalid("partition file/index length"));
        }
        input
            .seek(SeekFrom::Start(HEADER_BYTES as u64))
            .map_err(io_error)?;
        let mut bytes = vec![0; index_len];
        input.read_exact(&mut bytes).map_err(io_error)?;
        if crc32fast::hash(&bytes) != u32_at(&header, 24) {
            return Err(invalid("partition index CRC"));
        }
        let index: Index =
            serde_json::from_slice(&bytes).map_err(|e| Error::Encoding(e.to_string()))?;
        // Reuse U01 source/revision validation, including empty/conflicting IDs.
        Catalog::new(index.revision.clone(), index.sources.clone(), vec![])?;
        if index.cells.is_empty()
            || index.cells.len() > MAX_CELLS
            || index.sources.windows(2).any(|s| s[0].id >= s[1].id)
        {
            return Err(invalid("partition cell/source order/count"));
        }
        let payload_start = (HEADER_BYTES + index_len) as u64;
        let mut offset = 0_u64;
        let mut record_cells = BTreeMap::new();
        for (i, cell) in index.cells.iter().enumerate() {
            if (i > 0 && index.cells[i - 1].key >= cell.key)
                || cell.offset != offset
                || cell.compressed_bytes == 0
                || cell.raw_bytes == 0
                || cell.raw_bytes as usize > MAX_SCHEMA_JSON_BYTES
                || cell.record_ids.windows(2).any(|ids| ids[0] >= ids[1])
            {
                return Err(invalid("partition cell bounds/order"));
            }
            offset = offset
                .checked_add(u64::from(cell.compressed_bytes))
                .ok_or_else(|| invalid("partition offset overflow"))?;
            if offset > actual_total - payload_start {
                return Err(invalid("partition cell outside file"));
            }
            for id in &cell.record_ids {
                if index.sources.binary_search_by(|s| s.id.cmp(id)).is_ok()
                    || record_cells.insert(id.clone(), i).is_some()
                {
                    return Err(invalid("partition duplicate record/source index"));
                }
            }
        }
        if offset != actual_total - payload_start {
            return Err(invalid("partition trailing/unindexed bytes"));
        }
        Ok(Self {
            input,
            index,
            payload_start,
            record_cells,
        })
    }
    pub fn revision(&self) -> &str {
        &self.index.revision
    }
    pub fn sources(&self) -> &[Dataset] {
        &self.index.sources
    }
    pub fn cells(&self) -> &[CellIndex] {
        &self.index.cells
    }
    pub fn cell_for_record(&self, id: &CatalogId) -> Option<CellKey> {
        self.record_cells.get(id).map(|i| self.index.cells[*i].key)
    }
    pub fn read_cell(&mut self, key: CellKey) -> Result<Catalog, Error> {
        let i = self
            .index
            .cells
            .binary_search_by_key(&key, |e| e.key)
            .map_err(|_| invalid("unknown partition cell"))?;
        let cell = &self.index.cells[i];
        self.input
            .seek(SeekFrom::Start(self.payload_start + cell.offset))
            .map_err(io_error)?;
        let mut compressed = vec![0; cell.compressed_bytes as usize];
        self.input.read_exact(&mut compressed).map_err(io_error)?;
        if crc32fast::hash(&compressed) != cell.crc32 {
            return Err(invalid("partition payload CRC"));
        }
        let mut decoder = ZlibDecoder::new(compressed.as_slice());
        let mut raw = Vec::new();
        decoder
            .by_ref()
            .take(u64::from(cell.raw_bytes) + 1)
            .read_to_end(&mut raw)
            .map_err(io_error)?;
        if raw.len() != cell.raw_bytes as usize
            || decoder.total_in() != u64::from(cell.compressed_bytes)
        {
            return Err(invalid("partition decompressed length/trailing stream"));
        }
        let catalog = Catalog::decode(&raw)?;
        if catalog.revision() != self.index.revision
            || !catalog
                .records()
                .iter()
                .map(|r| r.id())
                .eq(cell.record_ids.iter())
            || catalog.datasets().iter().any(|s| {
                self.index
                    .sources
                    .binary_search_by(|t| t.id.cmp(&s.id))
                    .ok()
                    .is_none_or(|i| self.index.sources[i] != *s)
            })
        {
            return Err(invalid("partition payload/index/source mismatch"));
        }
        Ok(catalog)
    }
}
#[cfg(test)]
mod tests;
