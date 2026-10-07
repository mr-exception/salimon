extern crate diagnostics_after as after;
extern crate diagnostics_before as before;
use std::time::Duration;

macro_rules! step {
    ($api:ident, $diagnostics:ident, $i:ident) => {{
        use $api::{BodyDistance, DomainMetrics, FrameSample, GpuMemory, GpuTime};
        if $i % 47 == 0 {
            $diagnostics.toggle();
        }
        if $i % 61 == 0 {
            $diagnostics.reset_frame_window();
        }
        if $i % 53 == 0 {
            $diagnostics.record_memory_warning();
        }
        if $i % 71 == 0 {
            $diagnostics
                .set_scale_factor([f64::NAN, 0.0, 1.0, 1.5, 2.0, 3.0, 8.0][($i / 71) as usize % 7]);
        }
        let bodies = [
            BodyDistance {
                name: "Mars",
                distance_meters: 2_500_000.0,
            },
            BodyDistance {
                name: "Earth",
                distance_meters: ($i * 10) as f64,
            },
            BodyDistance {
                name: "invalid",
                distance_meters: -1.0,
            },
        ];
        $diagnostics.record_presented(
            FrameSample {
                frame_number: $i,
                frame_interval: Duration::from_micros(if $i % 11 == 0 {
                    0
                } else {
                    8_000 + ($i % 20) * 1000
                }),
                cpu_render_time: Duration::from_micros($i % 3 * 500),
                cpu_update_time: ($i % 2 == 0).then_some(Duration::from_micros(300)),
                gpu_time: match $i % 3 {
                    0 => GpuTime::Unsupported,
                    1 => GpuTime::Pending,
                    _ => GpuTime::Measured(Duration::from_micros(800)),
                },
                visible_objects: $i as u32,
                rendered_objects: 2,
                scene_draw_calls: 3,
                total_draw_calls: 4,
                gpu_memory: ($i % 2 == 0).then_some(GpuMemory {
                    allocated_bytes: 2_097_152,
                    reserved_bytes: 4_194_304,
                }),
            },
            DomainMetrics {
                camera_position: Some([1e12, -1e9, 12.0]),
                camera_altitude_meters: Some($i as f64),
                camera_phase: Some(if $i % 2 == 0 { " approach " } else { "  " }),
                camera_paused: Some($i % 2 == 0),
                player_position: Some([1000.0, 2000.0, 3000.0]),
                ship_position: Some([f64::NAN, 0.0, 0.0]),
                ship_velocity: Some([0.0, 1500.0, 0.0]),
                ship_speed_mps: Some($i as f64),
                thruster_percent: Some(($i % 110) as u8),
                nearby_bodies: &bodies,
            },
        );
    }};
}
fn main() {
    let mut old = before::Diagnostics::default();
    let mut new = after::Diagnostics::default();
    for i in 0..1000 {
        step!(before, old, i);
        step!(after, new, i);
        assert_eq!(old.is_visible(), new.is_visible(), "visibility at {i}");
        assert_eq!(old.overlay_text(), new.overlay_text(), "text at {i}");
        match (old.overlay(), new.overlay()) {
            (Some(a), Some(b)) => {
                assert_eq!(
                    (a.width, a.height, a.revision),
                    (b.width, b.height, b.revision),
                    "image metadata at {i}"
                );
                assert_eq!(a.rgba8, b.rgba8, "pixels at {i}");
            }
            (None, None) => {}
            _ => panic!("image availability differs at {i}"),
        }
    }
    println!(
        "1000 observation/state steps: identical visibility, text, dimensions, revisions and RGBA bytes"
    );
}
