//! Bounded observations for native terrain acceptance, never simulation authority.
use salimon_renderer::{GpuFrameTime, RenderStats};
use serde_json::{Value, json};
use std::{collections::VecDeque, time::Duration};
#[derive(Default)]
pub(crate) struct Observations {
    frames: VecDeque<[f64; 6]>,
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
            f64::from(stats.scene_draw_calls),
            f64::from(stats.total_draw_calls),
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
            "raw_frame_columns": ["interval_ms", "cpu_update_ms", "cpu_render_ms", "gpu_pass_ms", "scene_draw_calls", "total_draw_calls"],
            "raw_frames": self.frames,
            "gpu_allocator": self.gpu_memory.map(|m| json!({"allocated_bytes":m.allocated_bytes,"reserved_bytes":m.reserved_bytes})) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn raw_frames_preserve_spikes_missing_gpu_and_bounded_reset() {
        let mut observations = Observations::default();
        for i in 0..3601 {
            observations.record(
                Duration::from_millis(i),
                Duration::from_millis(2),
                RenderStats {
                    resource_batch: Default::default(),
                    cpu_render_time: Duration::from_millis(3),
                    gpu_frame_time: GpuFrameTime::Unsupported,
                    visible_objects: 0,
                    rendered_objects: 0,
                    scene_draw_calls: 7,
                    total_draw_calls: 9,
                    gpu_memory: None,
                },
                [1280, 800],
            );
        }
        let value = observations.snapshot();
        assert_eq!(value["samples"], 3600);
        assert_eq!(value["raw_frames"][0][0], 1.0);
        assert_eq!(value["raw_frames"][3599][0], 3600.0);
        assert!(value["raw_frames"][0][3].is_null());
        assert!(value["gpu_pass"].is_null());
        assert_eq!(value["raw_frames"][0][4], 7.0);
        assert_eq!(value["raw_frames"][0][5], 9.0);
        observations.clear();
        assert_eq!(observations.snapshot()["samples"], 0);
    }
}
