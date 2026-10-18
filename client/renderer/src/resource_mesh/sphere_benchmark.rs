use super::*;
use std::{io::Write, time::Instant};

pub(super) fn camera() -> CameraFrame {
    CameraFrame {
        position_meters: [1e12; 3],
        target_meters: [1e12, 1e12, 1e12 - 1.0],
        up: [0.0, 1.0, 0.0],
        vertical_fov_radians: 1.0,
        near_plane_meters: 0.1,
    }
}
pub(super) fn workload(count: usize, preview: bool, legacy: bool) -> Vec<ResourceMeshInstance> {
    (0..count + usize::from(preview))
        .map(|i| {
            // Matched 10 kg material, fixed camera/poses; old enlargement is deliberate.
            let density = [920.0, 2700.0, 3500.0][(i / 2) % 3];
            let radius = (3.0 * 10.0 / (4.0 * std::f64::consts::PI * density)).cbrt();
            ResourceMeshInstance {
                mesh: if legacy {
                    tests::ALL_MESHES[i % 6]
                } else {
                    ResourceMesh::Sphere
                },
                tint: if legacy {
                    [1.0; 4]
                } else {
                    [
                        [0.32, 0.68, 0.88, 1.0],
                        [0.45, 0.37, 0.28, 1.0],
                        [0.34, 0.29, 0.24, 1.0],
                    ][(i / 2) % 3]
                },
                center_meters: [
                    1e12 + (i % 20) as f64 * 0.21 - 2.0,
                    1e12 + (i / 20) as f64 * 0.21 - 2.5,
                    1e12 - 12.0,
                ],
                authored_mesh_scale: if legacy {
                    (10.0 / density).cbrt() * 5.0
                } else {
                    radius / 0.48
                },
                orientation_xyzw: [0.0, 0.0, 0.0, 1.0],
            }
        })
        .collect()
}

#[test]
fn sphere_workloads_preserve_one_batch_and_bounded_instance_records() {
    for count in [10, 100, 500] {
        for preview in [false, true] {
            let instances = workload(count, preview, false);
            let mut batches = std::array::from_fn(|_| InstanceBatch::default());
            prepare_instances(&mut batches, &instances, camera(), 1.0).unwrap();
            assert_eq!(batches.iter().filter(|b| !b.staging.is_empty()).count(), 1);
            assert_eq!(
                batches.iter().map(|b| b.staging.len()).sum::<usize>(),
                instances.len() * INSTANCE_BYTES
            );
            assert_eq!(ResourceMesh::Sphere.unit_vertices().len() / 3, 80);
        }
    }
}

#[test]
#[ignore = "opt-in fixed-camera legacy/sphere CPU preparation comparison, no GPU time"]
fn sphere_renderer_cpu_benchmark() {
    let mut output = std::fs::File::create(
        std::env::var("SALIMON_SPHERE_RENDER_REPORT").expect("set SALIMON_SPHERE_RENDER_REPORT"),
    )
    .unwrap();
    writeln!(output,"variant,count,preview,sample,prepare_ms,submitted_instances,batches,encoded_instance_bytes,submitted_triangles,scratch_capacity_bytes").unwrap();
    for legacy in [true, false] {
        for count in [10, 100, 500] {
            for preview in [false, true] {
                let instances = workload(count, preview, legacy);
                let mut batches = std::array::from_fn(|_| InstanceBatch::default());
                prepare_instances(&mut batches, &instances, camera(), 1.0).unwrap();
                for sample in 0..100 {
                    let start = Instant::now();
                    prepare_instances(&mut batches, &instances, camera(), 1.0).unwrap();
                    let ms = start.elapsed().as_secs_f64() * 1000.0;
                    let bytes: usize = batches.iter().map(|b| b.staging.len()).sum();
                    let draws = batches.iter().filter(|b| !b.staging.is_empty()).count();
                    let triangles: usize = instances
                        .iter()
                        .map(|i| i.mesh.unit_vertices().len() / 3)
                        .sum();
                    let capacity: usize = batches.iter().map(|b| b.staging.capacity()).sum();
                    assert_eq!(bytes, instances.len() * INSTANCE_BYTES);
                    if !legacy {
                        assert_eq!(draws, 1);
                        assert_eq!(triangles, instances.len() * 80);
                    }
                    writeln!(output,"{},{count},{preview},{sample},{ms},{},{draws},{bytes},{triangles},{capacity}",if legacy {"legacy-shards"} else {"sphere"},instances.len()).unwrap();
                }
            }
        }
    }
}
