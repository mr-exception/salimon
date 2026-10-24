//! Predictive partition streaming. All filesystem/decode work stays off the frame thread.
use crate::{
    Catalog, Error,
    coordinates::GalacticPosition,
    invalid,
    partitions::CellKey,
    reader::{Reader, Stats},
};
use std::{
    path::Path,
    sync::{Arc, mpsc},
    thread,
};

/// Presentation quality controls work/residency only; never canonical definitions.
#[derive(Clone, Copy, Debug)]
pub struct Budget {
    pub cache_bytes: usize,
    pub active_cells: usize,
    pub cells_per_poll: usize,
    pub lookahead_seconds: f64,
}
impl Default for Budget {
    fn default() -> Self {
        Self {
            cache_bytes: 16 * 1024 * 1024,
            active_cells: 16,
            cells_per_poll: 2,
            lookahead_seconds: 5.0,
        }
    }
}
#[derive(Clone)]
pub struct LoadedCell {
    pub key: CellKey,
    pub catalog: Arc<Catalog>,
    pub sky: Arc<Vec<crate::sky::StarSource>>,
}
struct Request {
    generation: u64,
    key: CellKey,
}
struct Reply {
    generation: u64,
    key: CellKey,
    result: Result<(Catalog, Vec<crate::sky::StarSource>), Error>,
    stats: Stats,
    resident: usize,
}
/// One request/result in flight. Stale results never become current after a jump.
/// The immutable manifest/index and at most active_cells decoded Catalogs are
/// separately bounded by the pack schema. Encoded cache capacity is exact.
pub struct Stream {
    budget: Budget,
    keys: Vec<CellKey>,
    wanted: Vec<CellKey>,
    active: Vec<LoadedCell>,
    generation: u64,
    pending: Option<(u64, CellKey)>,
    sender: Option<mpsc::SyncSender<Request>>,
    receiver: mpsc::Receiver<Reply>,
    worker: Option<thread::JoinHandle<()>>,
    pub last_error: Option<String>,
    pub stats: Stats,
    pub resident_encoded_bytes: usize,
}
impl Stream {
    pub fn open(path: &Path, budget: Budget) -> Result<Self, Error> {
        if budget.active_cells == 0
            || budget.active_cells > 64
            || budget.cells_per_poll == 0
            || budget.cells_per_poll > 8
            || !budget.lookahead_seconds.is_finite()
            || !(0.0..=60.0).contains(&budget.lookahead_seconds)
        {
            return Err(invalid("streaming budget"));
        }
        let mut reader = Reader::open(path, budget.cache_bytes)?;
        let keys = reader.manifest().cells.iter().map(|c| c.key).collect();
        let (sender, requests) = mpsc::sync_channel::<Request>(1);
        let (results, receiver) = mpsc::sync_channel(1);
        let worker = thread::spawn(move || {
            while let Ok(request) = requests.recv() {
                let result = reader.cell(request.key).and_then(|catalog| {
                    let has_sky = reader
                        .manifest()
                        .cells
                        .iter()
                        .any(|c| c.key == request.key && c.sidecars.contains_key("sky"));
                    let sky = if has_sky {
                        crate::sky::decode(&reader.sidecar(request.key, "sky")?, &catalog)?
                    } else {
                        vec![]
                    };
                    Ok((catalog, sky))
                });
                if results
                    .send(Reply {
                        generation: request.generation,
                        key: request.key,
                        result,
                        stats: reader.stats(),
                        resident: reader.resident_bytes(),
                    })
                    .is_err()
                {
                    break;
                }
            }
        });
        Ok(Self {
            budget,
            keys,
            wanted: Vec::new(),
            active: Vec::new(),
            generation: 0,
            pending: None,
            sender: Some(sender),
            receiver,
            worker: Some(worker),
            last_error: None,
            stats: Stats::default(),
            resident_encoded_bytes: 0,
        })
    }
    /// No IO, mutex acquisition, blocking send or thread join. Distances use
    /// integer cell deltas before f64, preserving local origins galaxy-wide.
    pub fn update(
        &mut self,
        position: GalacticPosition,
        velocity_mps: [f64; 3],
    ) -> Result<(), Error> {
        if !velocity_mps.into_iter().all(f64::is_finite) {
            return Err(invalid("stream velocity"));
        }
        let predicted =
            position.translated(velocity_mps.map(|v| v * self.budget.lookahead_seconds))?;
        let distance = |key: &CellKey, p: GalacticPosition| {
            let cells = p.cells();
            (0..3)
                .map(|i| (i128::from(key.galactic[i]) - i128::from(cells[i])).unsigned_abs())
                .sum::<u128>()
        };
        // Prefer current region; predicted regions follow. Stable key tie-breaks
        // make quality changes affect selection, never cell contents.
        let mut ranked: Vec<_> = self
            .keys
            .iter()
            .copied()
            .map(|k| {
                let now = distance(&k, position);
                let ahead = distance(&k, predicted);
                (now.min(ahead.saturating_add(1)), now, k)
            })
            .collect();
        ranked.sort_unstable();
        let wanted: Vec<_> = ranked
            .into_iter()
            .take(self.budget.active_cells)
            .map(|(_, _, k)| k)
            .collect();
        if wanted != self.wanted {
            self.generation = self
                .generation
                .checked_add(1)
                .ok_or_else(|| invalid("stream generation overflow"))?;
            self.wanted = wanted;
            self.active.retain(|c| self.wanted.contains(&c.key));
            self.last_error = None;
        }
        for _ in 0..self.budget.cells_per_poll {
            match self.receiver.try_recv() {
                Ok(reply) => {
                    self.pending = None;
                    self.stats = reply.stats;
                    self.resident_encoded_bytes = reply.resident;
                    if reply.generation == self.generation && self.wanted.contains(&reply.key) {
                        match reply.result {
                            Ok((catalog, sky)) => self.active.push(LoadedCell {
                                key: reply.key,
                                catalog: Arc::new(catalog),
                                sky: Arc::new(sky),
                            }),
                            Err(error) => {
                                self.last_error = Some(format!("{:?}: {error}", reply.key))
                            }
                        }
                    }
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    return Err(invalid("stream worker disconnected"));
                }
            }
        }
        // A failed required cell is visible and does not thrash disk every frame.
        // A changed request clears the failure and permits a retry.
        if self.pending.is_none()
            && self.last_error.is_none()
            && let Some(key) = self
                .wanted
                .iter()
                .find(|k| !self.active.iter().any(|c| c.key == **k))
                .copied()
        {
            let request = Request {
                generation: self.generation,
                key,
            };
            match self
                .sender
                .as_ref()
                .ok_or_else(|| invalid("stream closed"))?
                .try_send(request)
            {
                Ok(()) => self.pending = Some((self.generation, key)),
                Err(mpsc::TrySendError::Full(_)) => {}
                Err(mpsc::TrySendError::Disconnected(_)) => {
                    return Err(invalid("stream worker disconnected"));
                }
            }
        }
        Ok(())
    }
    pub fn active(&self) -> &[LoadedCell] {
        &self.active
    }
    pub fn wanted(&self) -> &[CellKey] {
        &self.wanted
    }
    pub fn ready(&self) -> bool {
        !self.wanted.is_empty()
            && self.last_error.is_none()
            && self.active.len() == self.wanted.len()
    }
}
impl Drop for Stream {
    fn drop(&mut self) {
        self.sender.take();
        // Disconnect the bounded result channel before joining, even if its
        // single slot is full. Only the current bounded disk read can delay exit.
        let (_, replacement) = mpsc::sync_channel(1);
        let receiver = std::mem::replace(&mut self.receiver, replacement);
        drop(receiver);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
#[cfg(test)]
mod tests;
