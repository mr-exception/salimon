//! Canonical sky presentation adapter; never writes scanner/chart or saves.
use salimon_renderer::{SkyScene, SkyStar};

pub(super) fn scene(
    camera: [f64; 3],
    elapsed: std::time::Duration,
    stream: Option<&salimon_universe::streaming::Stream>,
) -> Result<SkyScene, String> {
    let start = &salimon_world::canonical::START;
    let mut sky = SkyScene::default();
    let manifest = start.manifest();
    let a = manifest.physical.semi_major_axis_meters;
    // Published starting-system adapter currently has fixed body-local axes and illumination.
    // Do not invent orbital phase: nearby pack stars require an explicit galactic observer below.
    let relative = [-camera[0], a - camera[1], -camera[2]];
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
        let observer = manifest
            .distance
            .system_position
            .translated(camera)
            .map_err(|e| e.to_string())?;
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
                    star.epoch
                        .advance_nanos(
                            i64::try_from(elapsed.as_nanos()).map_err(|_| "sky elapsed range")?,
                        )
                        .map_err(|e| e.to_string())?,
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

#[cfg(test)]
mod tests {
    #[test]
    fn canonical_sky_uses_published_radius_without_stream_or_discoveries() {
        let p = salimon_world::canonical::START.manifest();
        let sky = super::scene([0.; 3], std::time::Duration::ZERO, None).unwrap();
        assert_eq!(sky.stars.len(), 1);
        assert_eq!(sky.stars[0].direction, [0., 1., 0.]);
        let angle =
            (p.physical.star_radius_meters / p.physical.semi_major_axis_meters).asin() as f32;
        assert_eq!(sky.stars[0].angular_radius_radians, angle);
    }
}
