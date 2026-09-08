use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

const QUERY_COUNT: u32 = 2;
const QUERY_BUFFER_SIZE: u64 = QUERY_COUNT as u64 * wgpu::QUERY_SIZE as u64;
const READBACK_SLOT_COUNT: usize = 3;

#[derive(Debug)]
struct ReadbackSlot {
    buffer: wgpu::Buffer,
    pending: bool,
}

#[derive(Debug)]
struct TimingResult {
    slot_index: usize,
    duration: Option<Duration>,
}

/// Non-blocking timestamp-query ring for completed GPU render passes.
pub(crate) struct GpuTimer {
    query_set: wgpu::QuerySet,
    resolve_buffer: wgpu::Buffer,
    readback_slots: Vec<ReadbackSlot>,
    sender: Sender<TimingResult>,
    receiver: Receiver<TimingResult>,
    timestamp_period_ns: f32,
    latest: Option<Duration>,
}

impl GpuTimer {
    pub(crate) fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let query_set = device.create_query_set(&wgpu::QuerySetDescriptor {
            label: Some("Salimon frame timestamp queries"),
            ty: wgpu::QueryType::Timestamp,
            count: QUERY_COUNT,
        });
        let resolve_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Salimon timestamp resolve buffer"),
            size: QUERY_BUFFER_SIZE,
            usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let readback_slots = (0..READBACK_SLOT_COUNT)
            .map(|index| ReadbackSlot {
                buffer: device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(match index {
                        0 => "Salimon timestamp readback 0",
                        1 => "Salimon timestamp readback 1",
                        _ => "Salimon timestamp readback 2",
                    }),
                    size: QUERY_BUFFER_SIZE,
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                    mapped_at_creation: false,
                }),
                pending: false,
            })
            .collect();
        let (sender, receiver) = mpsc::channel();

        Self {
            query_set,
            resolve_buffer,
            readback_slots,
            sender,
            receiver,
            timestamp_period_ns: queue.get_timestamp_period(),
            latest: None,
        }
    }

    pub(crate) fn poll(&mut self, device: &wgpu::Device) {
        let _ = device.poll(wgpu::PollType::Poll);
        while let Ok(result) = self.receiver.try_recv() {
            if let Some(slot) = self.readback_slots.get_mut(result.slot_index) {
                slot.pending = false;
            }
            if let Some(duration) = result.duration {
                self.latest = Some(duration);
            }
        }
    }

    pub(crate) fn acquire_slot(&mut self) -> Option<usize> {
        let slot_index = self.readback_slots.iter().position(|slot| !slot.pending)?;
        self.readback_slots[slot_index].pending = true;
        Some(slot_index)
    }

    pub(crate) fn timestamp_writes(&self) -> wgpu::RenderPassTimestampWrites<'_> {
        wgpu::RenderPassTimestampWrites {
            query_set: &self.query_set,
            beginning_of_pass_write_index: Some(0),
            end_of_pass_write_index: Some(1),
        }
    }

    pub(crate) fn resolve_and_map(&self, encoder: &mut wgpu::CommandEncoder, slot_index: usize) {
        let readback_buffer = self.readback_slots[slot_index].buffer.clone();
        encoder.resolve_query_set(&self.query_set, 0..QUERY_COUNT, &self.resolve_buffer, 0);
        encoder.copy_buffer_to_buffer(
            &self.resolve_buffer,
            0,
            &readback_buffer,
            0,
            QUERY_BUFFER_SIZE,
        );

        let callback_buffer = readback_buffer.clone();
        let sender = self.sender.clone();
        let timestamp_period_ns = self.timestamp_period_ns;
        encoder.map_buffer_on_submit(
            &readback_buffer,
            wgpu::MapMode::Read,
            ..,
            move |mapping_result| {
                let duration = if mapping_result.is_ok() {
                    let parsed = callback_buffer.get_mapped_range(..).ok().and_then(|view| {
                        let bytes: &[u8] = &view;
                        let start = u64::from_ne_bytes(bytes.get(0..8)?.try_into().ok()?);
                        let end = u64::from_ne_bytes(bytes.get(8..16)?.try_into().ok()?);
                        Some(duration_from_ticks(
                            end.wrapping_sub(start),
                            timestamp_period_ns,
                        ))
                    });
                    callback_buffer.unmap();
                    parsed
                } else {
                    None
                };
                let _ = sender.send(TimingResult {
                    slot_index,
                    duration,
                });
            },
        );
    }

    pub(crate) const fn latest(&self) -> Option<Duration> {
        self.latest
    }
}

fn duration_from_ticks(ticks: u64, timestamp_period_ns: f32) -> Duration {
    let nanoseconds = (ticks as f64 * f64::from(timestamp_period_ns)).max(0.0);
    Duration::from_secs_f64(nanoseconds / 1_000_000_000.0)
}

#[cfg(test)]
mod tests {
    use super::duration_from_ticks;
    use std::time::Duration;

    #[test]
    fn timestamp_ticks_use_the_adapter_period() {
        assert_eq!(duration_from_ticks(2_000, 2.5), Duration::from_micros(5));
    }
}
