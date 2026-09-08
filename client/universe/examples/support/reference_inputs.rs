//! Frozen-source test adapter and production U02 workloads, not a U08 importer.
use super::universe::{CatalogId, Epoch, TimeScale, coordinates::*, reference::Sample};
use std::error::Error;
pub fn samples() -> Result<Vec<Sample>, Box<dyn Error>> {
    let raw: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../catalogs/sources/fixtures/jpl-earth-j2000.json"
    ))?;
    let result = raw["result"].as_str().ok_or("missing frozen JPL result")?;
    if !result.contains("Output units    : KM-S") || !result.contains("Reference frame : ICRF") {
        return Err("frozen JPL context mismatch".into());
    }
    let block = result
        .split("$$SOE\n")
        .nth(1)
        .and_then(|s| s.split("$$EOE").next())
        .ok_or("missing frozen JPL block")?;
    let lines: Vec<_> = block.lines().filter(|s| !s.trim().is_empty()).collect();
    if lines.len() != 6 {
        return Err("frozen JPL sample count".into());
    }
    fn vector(line: &str, labels: [&str; 3]) -> Result<[f64; 3], Box<dyn Error>> {
        // Horizons permits blanks after '=', so split by the next component label.
        let mut values = [0.; 3];
        for i in 0..3 {
            let tail = line
                .split_once(labels[i])
                .ok_or("missing vector component")?
                .1;
            let tail = tail
                .strip_prefix('=')
                .ok_or("missing vector equals")?
                .trim_start();
            let token = tail
                .split_whitespace()
                .next()
                .ok_or("missing vector number")?;
            values[i] = token.parse::<f64>()? * 1000.0;
        }
        Ok(values)
    }
    let mut out = Vec::new();
    for i in 0..2 {
        let jd = ["2451544.500000000", "2451545.500000000"][i];
        if !lines[i * 3].starts_with(jd) || !lines[i * 3].ends_with("TDB ") {
            return Err("frozen JPL epoch mismatch".into());
        }
        out.push(Sample {
            id: format!("earth-jpl-{}", i + 1),
            frame: "ICRF-SSB-geometric".into(),
            epoch: Epoch::calendar([2000, 1, i as i32 + 1], [0, 0, 0, 0], TimeScale::Tdb)?,
            position_meters: vector(lines[i * 3 + 1], ["X ", "Y ", "Z "])?,
            velocity_mps: vector(lines[i * 3 + 2], ["VX", "VY", "VZ"])?,
        });
    }
    let epoch = Epoch::calendar([2000, 1, 1], [12, 0, 0, 0], TimeScale::Tdb)?;
    let id = |key: &str| CatalogId::from_source("fixture", key);
    let system = id("system")?;
    let body = FrameId::Body(id("body")?);
    let local = FrameId::Local(id("local")?);
    let anchor = SystemAnchor {
        system: system.clone(),
        epoch,
        position: GalacticPosition::new([1_000_000; 3], [100., 200., 300.])?,
        velocity_mps: [3., -4., 12.],
        rotation: Rotation::IDENTITY,
    };
    let tree = FrameTree::new(
        anchor.clone(),
        vec![
            FrameLink {
                child: body.clone(),
                parent: FrameId::System(system),
                epoch,
                origin_meters: [0.; 3],
                velocity_mps: [0.; 3],
                rotation: Rotation::IDENTITY,
                angular_velocity_rps: [0.; 3],
            },
            FrameLink {
                child: local.clone(),
                parent: body.clone(),
                epoch,
                origin_meters: [10., 20., 30.],
                velocity_mps: [3., 4., 5.],
                rotation: Rotation::new([[0., 1., 0.], [-1., 0., 0.], [0., 0., 1.]])?,
                angular_velocity_rps: [0., 0., 2.],
            },
        ],
    )?;
    let transformed = tree.transform(
        &local,
        &body,
        State {
            meters: [2., 0., 0.],
            meters_per_second: [1., 0., 0.],
        },
        epoch,
    )?;
    out.push(Sample {
        id: "local-rotating".into(),
        frame: "fixture-body".into(),
        epoch,
        position_meters: transformed.meters,
        velocity_mps: transformed.meters_per_second,
    });
    let camera = GalacticPosition::new([1_000_000; 3], [CELL_METERS / 2. - 1., 0., 0.])?;
    let position = camera.translated([0.000244140625, 0., 0.])?;
    out.push(Sample {
        id: "distant-submillimeter".into(),
        frame: "galactic-camera-relative".into(),
        epoch,
        position_meters: position.camera_relative_f32(camera)?.map(f64::from),
        velocity_mps: [0.; 3],
    });
    let later = Epoch::calendar([2000, 1, 3], [12, 0, 0, 0], TimeScale::Tdb)?;
    out.push(Sample {
        id: "interstellar-proper-motion".into(),
        frame: "galactic-initial-relative".into(),
        epoch: later,
        position_meters: anchor
            .position_at(later)?
            .relative_meters(anchor.position)?,
        velocity_mps: anchor.velocity_mps,
    });
    Ok(out)
}
