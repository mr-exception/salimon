//! Two independently verified slots. Publishing never destroys the newest valid slot.
use crate::{MAX_SAVE_BYTES, SaveError, invalid};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const MAGIC: &[u8; 8] = b"SALICK01";
const HEADER: usize = 28;

/// Implementations must publish complete bytes or leave the other slot intact.
pub trait CheckpointStore {
    fn read(&self, slot: usize) -> Result<Option<Vec<u8>>, SaveError>;
    fn publish(&mut self, slot: usize, bytes: &[u8]) -> Result<(), SaveError>;
}

pub struct Recovery {
    pub bytes: Option<Vec<u8>>,
    pub recovered: bool,
}

pub struct SaveManager<S> {
    store: S,
    current: Option<(usize, u64)>,
}
impl<S: CheckpointStore> SaveManager<S> {
    pub fn new(store: S) -> Self {
        Self {
            store,
            current: None,
        }
    }
    /// Domain validation is required in addition to the storage integrity check.
    pub fn recover(
        &mut self,
        validate: impl Fn(&[u8]) -> Result<(), SaveError>,
    ) -> Result<Recovery, SaveError> {
        let mut valid = Vec::new();
        let mut damaged = false;
        for slot in 0..2 {
            match self.store.read(slot) {
                Ok(None) => (),
                Ok(Some(bytes)) => match unpack(&bytes).and_then(|(generation, payload)| {
                    validate(payload)?;
                    Ok((generation, payload.to_vec()))
                }) {
                    Ok((generation, payload)) => valid.push((generation, slot, payload)),
                    // Never overwrite a save from a newer game with older state.
                    Err(e @ SaveError::UnsupportedVersion(_)) => return Err(e),
                    Err(_) => damaged = true,
                },
                Err(e) => return Err(e),
            }
        }
        valid.sort_by_key(|entry| entry.0);
        if valid.len() == 2 && valid[0].0 == valid[1].0 {
            return Err(invalid("ambiguous checkpoint generations"));
        }
        if let Some((generation, slot, bytes)) = valid.pop() {
            self.current = Some((slot, generation));
            Ok(Recovery {
                bytes: Some(bytes),
                recovered: damaged,
            })
        } else {
            self.current = None;
            Ok(Recovery {
                bytes: None,
                recovered: damaged,
            })
        }
    }
    pub fn checkpoint(
        &mut self,
        payload: &[u8],
        validate: impl Fn(&[u8]) -> Result<(), SaveError>,
    ) -> Result<(), SaveError> {
        if payload.len() > MAX_SAVE_BYTES {
            return Err(invalid("checkpoint too large"));
        }
        validate(payload)?;
        let (slot, generation) = self
            .current
            .map_or((0, 1), |(slot, g)| (1 - slot, g.saturating_add(1)));
        if self.current.is_some_and(|(_, g)| g == u64::MAX) {
            return Err(invalid("checkpoint generation exhausted"));
        }
        let bytes = pack(generation, payload);
        self.store.publish(slot, &bytes)?;
        let readback = self
            .store
            .read(slot)?
            .ok_or_else(|| invalid("checkpoint disappeared"))?;
        if readback != bytes {
            return Err(invalid("checkpoint readback mismatch"));
        }
        let (_, restored) = unpack(&readback)?;
        validate(restored)?;
        self.current = Some((slot, generation));
        Ok(())
    }
}

fn checksum(bytes: &[u8]) -> u32 {
    // IEEE CRC32 detects accidental corruption; this is not an authenticity claim.
    let mut crc = !0u32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb88320 & 0u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}
fn pack(generation: u64, payload: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(HEADER + payload.len());
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&generation.to_le_bytes());
    bytes.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    let mut checked = bytes[8..].to_vec();
    checked.extend_from_slice(payload);
    bytes.extend_from_slice(&checksum(&checked).to_le_bytes());
    bytes.extend_from_slice(payload);
    bytes
}
fn unpack(bytes: &[u8]) -> Result<(u64, &[u8]), SaveError> {
    if bytes.len() < HEADER || bytes.len() > MAX_SAVE_BYTES + HEADER || &bytes[..8] != MAGIC {
        return Err(invalid("checkpoint header/size"));
    }
    // Fixed-size ranges were checked above.
    let generation =
        u64::from_le_bytes(bytes[8..16].try_into().map_err(|_| invalid("generation"))?);
    let length = u64::from_le_bytes(bytes[16..24].try_into().map_err(|_| invalid("length"))?);
    let crc = u32::from_le_bytes(bytes[24..28].try_into().map_err(|_| invalid("checksum"))?);
    let payload = &bytes[HEADER..];
    let mut checked = bytes[8..24].to_vec();
    checked.extend_from_slice(payload);
    if generation == 0 || length != payload.len() as u64 || checksum(&checked) != crc {
        return Err(invalid("checkpoint integrity"));
    }
    Ok((generation, payload))
}

pub struct NativeStore {
    directory: PathBuf,
    _lease: std::fs::File,
}
impl NativeStore {
    pub fn open(directory: impl Into<PathBuf>) -> Result<Self, SaveError> {
        let directory = directory.into();
        std::fs::create_dir_all(&directory)
            .map_err(|e| io_error("create save directory", &directory, e))?;
        let path = directory.join("active-run.lock");
        let lease = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .map_err(|e| io_error("open save lock", &path, e))?;
        lease.try_lock().map_err(|e| {
            invalid(format!(
                "lock {} (another game may be running): {e}",
                path.display()
            ))
        })?;
        Ok(Self {
            directory,
            _lease: lease,
        })
    }
    fn path(&self, slot: usize) -> PathBuf {
        self.directory.join(format!("checkpoint-{slot}.sav"))
    }
}
fn io_error(operation: &str, path: &Path, error: std::io::Error) -> SaveError {
    invalid(format!("{operation} {}: {error}", path.display()))
}
impl CheckpointStore for NativeStore {
    fn read(&self, slot: usize) -> Result<Option<Vec<u8>>, SaveError> {
        let path = self.path(slot);
        let file = match std::fs::File::open(&path) {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(io_error("read", &path, e)),
        };
        let mut bytes = Vec::new();
        file.take((MAX_SAVE_BYTES + HEADER + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|e| io_error("read", &path, e))?;
        Ok(Some(bytes))
    }
    fn publish(&mut self, slot: usize, bytes: &[u8]) -> Result<(), SaveError> {
        std::fs::create_dir_all(&self.directory)
            .map_err(|e| io_error("create directory", &self.directory, e))?;
        let destination = self.path(slot);
        let temporary = self.directory.join(format!("checkpoint-{slot}.tmp"));
        let mut file = std::fs::File::create(&temporary)
            .map_err(|e| io_error("create temporary", &temporary, e))?;
        file.write_all(bytes)
            .and_then(|()| file.sync_all())
            .map_err(|e| io_error("flush temporary", &temporary, e))?;
        drop(file);
        // Windows rename cannot replace an existing file. Only remove the inactive
        // slot; the other verified generation survives every interruption here.
        #[cfg(windows)]
        if destination.exists() {
            std::fs::remove_file(&destination)
                .map_err(|e| io_error("remove inactive", &destination, e))?;
        }
        std::fs::rename(&temporary, &destination)
            .map_err(|e| io_error("publish", &destination, e))?;
        #[cfg(unix)]
        std::fs::File::open(&self.directory)
            .and_then(|f| f.sync_all())
            .map_err(|e| io_error("flush directory", &self.directory, e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Default)]
    struct Memory {
        slots: [Option<Vec<u8>>; 2],
        fail: bool,
        partial: bool,
    }
    impl CheckpointStore for Memory {
        fn read(&self, slot: usize) -> Result<Option<Vec<u8>>, SaveError> {
            Ok(self.slots[slot].clone())
        }
        fn publish(&mut self, slot: usize, bytes: &[u8]) -> Result<(), SaveError> {
            if self.partial {
                self.slots[slot] = Some(bytes[..10].to_vec());
            }
            if self.fail || self.partial {
                return Err(invalid("injected disk full/interruption"));
            }
            self.slots[slot] = Some(bytes.to_vec());
            Ok(())
        }
    }
    fn validate(bytes: &[u8]) -> Result<(), SaveError> {
        if bytes.is_empty() {
            Err(invalid("empty"))
        } else {
            Ok(())
        }
    }
    #[test]
    fn no_valid_checkpoint_warns_but_future_version_never_silently_reseeds() {
        let mut manager = SaveManager::new(Memory::default());
        manager.store.slots[0] = Some(b"partial".to_vec());
        let recovery = manager.recover(validate).unwrap();
        assert!(recovery.bytes.is_none() && recovery.recovered);
        manager.checkpoint(b"first", validate).unwrap();
        manager.store.slots[1] = Some(pack(2, b"future"));
        let check_version = |b: &[u8]| {
            if b == b"future" {
                Err(SaveError::UnsupportedVersion(2))
            } else {
                validate(b)
            }
        };
        assert!(matches!(
            manager.recover(check_version),
            Err(SaveError::UnsupportedVersion(2))
        ));
        manager.store.slots[1] = Some(pack(2, b""));
        assert_eq!(manager.recover(validate).unwrap().bytes.unwrap(), b"first");
        // A flipped generation byte must fail integrity too.
        manager.store.slots[1] = Some(pack(2, b"second"));
        manager.store.slots[1].as_mut().unwrap()[8] ^= 4;
        assert_eq!(manager.recover(validate).unwrap().bytes.unwrap(), b"first");
        assert!(
            manager
                .checkpoint(&vec![0; MAX_SAVE_BYTES + 1], validate)
                .is_err()
        );
    }
    #[test]
    fn interrupted_write_and_corruption_preserve_verified_fallback() {
        let mut manager = SaveManager::new(Memory::default());
        assert!(manager.recover(validate).unwrap().bytes.is_none());
        manager.checkpoint(b"first", validate).unwrap();
        manager.store.partial = true;
        assert!(manager.checkpoint(b"second", validate).is_err());
        let recovery = manager.recover(validate).unwrap();
        assert_eq!(recovery.bytes.unwrap(), b"first");
        assert!(recovery.recovered);
        manager.store.partial = false;
        manager.checkpoint(b"second", validate).unwrap();
        manager.store.slots[1].as_mut().unwrap()[HEADER] ^= 1;
        assert_eq!(manager.recover(validate).unwrap().bytes.unwrap(), b"first");
        manager.checkpoint(b"third", validate).unwrap();
        assert_eq!(manager.recover(validate).unwrap().bytes.unwrap(), b"third");
    }
    #[test]
    fn malformed_payload_and_failed_publish_do_not_replace_current() {
        let mut manager = SaveManager::new(Memory::default());
        manager.checkpoint(b"first", validate).unwrap();
        assert!(manager.checkpoint(b"", validate).is_err());
        manager.store.fail = true;
        assert!(manager.checkpoint(b"second", validate).is_err());
        assert_eq!(manager.recover(validate).unwrap().bytes.unwrap(), b"first");
        assert_eq!(checksum(b"123456789"), 0xcbf43926);
        assert!(unpack(&pack(0, b"invalid")).is_err());
    }
    #[test]
    fn native_crash_restart_ignores_temporary_and_chooses_generation_not_clock() {
        let dir =
            std::env::temp_dir().join(format!("salimon-checkpoint-test-{}", std::process::id()));
        let mut manager = SaveManager::new(NativeStore::open(&dir).unwrap());
        manager.checkpoint(b"first", validate).unwrap();
        manager.checkpoint(b"second", validate).unwrap();
        std::fs::write(dir.join("checkpoint-0.tmp"), b"unfinished").unwrap();
        assert!(NativeStore::open(&dir).is_err());
        drop(manager);
        let mut restarted = SaveManager::new(NativeStore::open(&dir).unwrap());
        assert_eq!(
            restarted.recover(validate).unwrap().bytes.unwrap(),
            b"second"
        );
        restarted.checkpoint(b"third", validate).unwrap();
        assert_eq!(
            restarted.recover(validate).unwrap().bytes.unwrap(),
            b"third"
        );
        std::fs::remove_dir_all(dir).unwrap();
    }
}
