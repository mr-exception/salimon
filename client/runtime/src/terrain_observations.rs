//! Bounded observations for native terrain acceptance, never simulation authority.
use salimon_renderer::{GpuFrameTime, RenderStats};
use serde_json::{Value, json};
use std::{collections::VecDeque, time::Duration};
#[derive(Default)]
pub(crate) struct Observations {
    frames: VecDeque<[f64; 4]>,
    pub(crate) gpu_memory: Option<salimon_renderer::GpuMemoryMetrics>,
    pub(crate) size: [u32; 2],
}
impl Observations {
    pub(crate) fn record(
        &mut self,
        interval: Duration,
        update: Duration,
        stats: RenderStats,
        size: [u32; 2],
    ) {
        if self.frames.len() == 3600 {
            self.frames.pop_front();
        }
        let gpu = match stats.gpu_frame_time {
            GpuFrameTime::Measured(t) => t.as_secs_f64() * 1000.0,
            _ => f64::NAN,
        };
        self.frames.push_back([
            interval.as_secs_f64() * 1000.0,
            update.as_secs_f64() * 1000.0,
            stats.cpu_render_time.as_secs_f64() * 1000.0,
            gpu,
        ]);
        self.gpu_memory = stats.gpu_memory;
        self.size = size;
    }
    pub(crate) fn clear(&mut self) {
        self.frames.clear();
    }
    pub(crate) fn snapshot(&self) -> Value {
        let field = |index: usize| {
            let mut values: Vec<_> = self
                .frames
                .iter()
                .map(|f| f[index])
                .filter(|v| v.is_finite())
                .collect();
            if values.is_empty() {
                return json!(null);
            }
            values.sort_by(f64::total_cmp);
            let mean = values.iter().sum::<f64>() / values.len() as f64;
            json!({ "samples": values.len(), "mean_ms": mean, "p95_ms": values[(values.len()*95/100).min(values.len()-1)],
                "max_ms": values.last(), "min_ms": values.first() })
        };
        json!({ "drawable_pixels": self.size, "samples": self.frames.len(),
            "presented_interval": field(0), "cpu_update": field(1), "cpu_render": field(2), "gpu_pass": field(3),
            "gpu_allocator": self.gpu_memory.map(|m| json!({"allocated_bytes":m.allocated_bytes,"reserved_bytes":m.reserved_bytes})) })
    }
}
