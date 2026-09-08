//! Canonical sky presentation adapter; never writes scanner/chart or saves.
use salimon_renderer::{SkyScene, SkyStar};

pub(super) fn scene(
    camera: [f64; 3],
    epoch: salimon_universe::Epoch,
    stream: Option<&salimon_universe::streaming::Stream>,
) -> Result<SkyScene, String> {
    let start = &salimon_world::canonical::START;
    let mut sky = SkyScene::default();
    let manifest = start.manifest();
    let star = host_position(epoch)?;
    let relative = std::array::from_fn::<_, 3, _>(|i| star[i] - camera[i]);
    let distance = relative[0].hypot(relative[1]).hypot(relative[2]);
    let radius = manifest.physical.star_radius_meters;
    if distance > radius {
        sky.stars.push(SkyStar {
            direction: relative.map(|x| (x / distance) as f32),
            angular_radius_radians: (radius / distance).asin() as f32,
            color: [1., 0.94, 0.82],
            brightness: 1.,
        });
    }
    if let Some(stream) = stream {
        let observer = salimon_universe::starting_motion::inertial(epoch, camera, [0.; 3])
            .map_err(|e| e.to_string())?
            .0;
        let mut sources = std::collections::BTreeMap::new();
        for cell in stream.active() {
            for star in cell.sky.iter() {
                sources.entry(star.id.clone()).or_insert(star);
            }
        }
        let mut visible = Vec::new();
        for star in sources.values() {
            if let Some(p) = star
                .project(
                    observer,
                    // Unknown velocity holds the published reference position.
                    // Moving sources require the same explicit time scale;
                    // project rejects TT/TDB mixing instead of fabricating conversion.
                    if star.velocity_mps.is_some() {
                        epoch
                    } else {
                        star.epoch
                    },
                )
                .map_err(|e| e.to_string())?
            {
                visible.push((
                    star.id.clone(),
                    SkyStar {
                        direction: p.direction.map(|x| x as f32),
                        angular_radius_radians: p.angular_radius_radians as f32,
                        color: p.color,
                        brightness: p.brightness,
                    },
                ));
            }
        }
        visible.sort_by(|a, b| {
            b.1.brightness
                .total_cmp(&a.1.brightness)
                .then_with(|| a.0.cmp(&b.0))
        });
        sky.stars.extend(
            visible
                .into_iter()
                .take(salimon_renderer::MAX_SKY_STARS - 1)
                .map(|(_, star)| star),
        );
    }
    Ok(sky)
}

/// Nacre's centre in the nonrotating, translating Vesper frame, shared by
/// point lighting and the host disk. No separate visual orbit is generated.
pub(super) fn host_position(epoch: salimon_universe::Epoch) -> Result<[f64; 3], String> {
    salimon_universe::starting_motion::planet(epoch)
        .map(|planet| planet.position_meters.map(|x| -x))
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    #[test]
    fn canonical_sky_uses_published_radius_without_stream_or_discoveries() {
        let p = salimon_world::canonical::START.manifest();
        let sky = super::scene(
            [0.; 3],
            salimon_universe::starting_motion::DEFINITION.reference_epoch,
            None,
        )
        .unwrap();
        assert_eq!(sky.stars.len(), 1);
        assert_eq!(sky.stars[0].direction, [-1., 0., 0.]);
        let angle =
            (p.physical.star_radius_meters / p.physical.semi_major_axis_meters).asin() as f32;
        assert_eq!(sky.stars[0].angular_radius_radians, angle);
        let epoch = salimon_universe::starting_motion::Clock::at(1_793_404_800)
            .unwrap()
            .epoch;
        let camera = [123., 456., 789.];
        let sky = super::scene(camera, epoch, None).unwrap();
        let star = super::host_position(epoch).unwrap();
        let relative = std::array::from_fn::<_, 3, _>(|i| star[i] - camera[i]);
        let distance = salimon_math::length(relative);
        assert_eq!(
            sky.stars[0].direction,
            relative.map(|v| (v / distance) as f32)
        );
        assert_ne!(sky.stars[0].direction, [-1., 0., 0.]);
    }
}
