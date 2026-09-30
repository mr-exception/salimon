//! Original, deterministic material-demo textures. No external imagery or assets.

pub(crate) const WIDTH: u32 = 512;
pub(crate) const HEIGHT: u32 = 256;
pub(crate) const LAYERS: u32 = 7;
pub(crate) const MIP_COUNT: u32 = 10;

/// Samples a continuous 3D field on the sphere so longitude seams and poles agree.
fn noise(point: [f64; 3], seed: u32) -> f64 {
    let cell = point.map(|x| x.floor() as i32);
    let blend = point.map(|x| {
        let t = x - x.floor();
        t * t * (3.0 - 2.0 * t)
    });
    let mut value = 0.0;
    for z in 0..2 {
        for y in 0..2 {
            for x in 0..2 {
                let offsets = [x, y, z];
                let mut hash = seed;
                let mut weight = 1.0;
                for axis in 0..3 {
                    hash ^= (cell[axis].wrapping_add(offsets[axis]) as u32)
                        .wrapping_mul([0x8da6b343, 0xd8163841, 0xcb1ab31f][axis]);
                    weight *= if offsets[axis] == 0 {
                        1.0 - blend[axis]
                    } else {
                        blend[axis]
                    };
                }
                hash ^= hash >> 16;
                hash = hash.wrapping_mul(0x7feb352d);
                hash ^= hash >> 15;
                value += weight * f64::from(hash) / f64::from(u32::MAX);
            }
        }
    }
    value
}

fn albedo(normal: [f64; 3], layer: u32) -> [u8; 4] {
    let seed = layer.wrapping_mul(734_287).wrapping_add(91);
    let broad = noise(normal.map(|x| x * 4.0 + 7.0), seed);
    let medium = noise(normal.map(|x| x * 18.0 + 3.0), seed + 1);
    let fine = noise(normal.map(|x| x * 70.0), seed + 2);
    let variation = 0.55 * broad + 0.3 * medium + 0.15 * fine;
    let (dark, light) = match layer {
        0 => ([0.25, 0.23, 0.21], [0.65, 0.60, 0.53]),
        1 => ([0.38, 0.18, 0.06], [0.85, 0.63, 0.27]),
        2 if broad < 0.50 => ([0.02, 0.09, 0.27], [0.05, 0.30, 0.57]),
        2 => ([0.10, 0.23, 0.12], [0.47, 0.53, 0.25]),
        3 => ([0.24, 0.25, 0.27], [0.70, 0.69, 0.63]),
        4 => ([0.32, 0.07, 0.035], [0.80, 0.32, 0.12]),
        _ => ([0.95, 0.31, 0.035], [1.0, 0.86, 0.22]),
    };
    let t = ((variation - 0.25) * 2.0).clamp(0.0, 1.0);
    let rgb: [u8; 3] =
        std::array::from_fn(|i| ((dark[i] + (light[i] - dark[i]) * t) * 255.0) as u8);
    [rgb[0], rgb[1], rgb[2], 255]
}

pub(crate) fn mip_chain(layer: u32) -> Vec<(u32, u32, Vec<u8>)> {
    let mut pixels = Vec::with_capacity((WIDTH * HEIGHT * 4) as usize);
    for y in 0..HEIGHT {
        let latitude = (0.5 - (f64::from(y) + 0.5) / f64::from(HEIGHT)) * std::f64::consts::PI;
        for x in 0..WIDTH {
            let longitude = ((f64::from(x) + 0.5) / f64::from(WIDTH) - 0.5) * std::f64::consts::TAU;
            if layer == 6 {
                // A periodic torus field gives the local detail tile matching edges.
                let v = (f64::from(y) + 0.5) / f64::from(HEIGHT) * std::f64::consts::TAU;
                let point = [
                    longitude.cos() + v.sin(),
                    longitude.sin() + v.cos(),
                    v.sin(),
                ];
                let value = 0.35 + 0.3 * noise(point.map(|x| x * 12.0), 731);
                let gray = (value * 255.0) as u8;
                pixels.extend_from_slice(&[gray, gray, gray, 255]);
                continue;
            }
            pixels.extend_from_slice(&albedo(
                [
                    latitude.cos() * longitude.cos(),
                    latitude.sin(),
                    latitude.cos() * longitude.sin(),
                ],
                layer,
            ));
        }
    }
    let mut chain = vec![(WIDTH, HEIGHT, pixels)];
    while chain.len() < MIP_COUNT as usize {
        let (width, height, source) = chain.last().expect("base mip exists");
        let next_width = (width / 2).max(1);
        let next_height = (height / 2).max(1);
        let mut mip = Vec::with_capacity((next_width * next_height * 4) as usize);
        for y in 0..next_height {
            for x in 0..next_width {
                for channel in 0..4 {
                    let mut total = 0_u32;
                    for dy in 0..2 {
                        for dx in 0..2 {
                            let sx = (x * 2 + dx).min(width - 1);
                            let sy = (y * 2 + dy).min(height - 1);
                            total += u32::from(source[((sy * width + sx) * 4 + channel) as usize]);
                        }
                    }
                    mip.push((total / 4) as u8);
                }
            }
        }
        chain.push((next_width, next_height, mip));
    }
    chain
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generic_materials_are_distinct_and_mip_chains_end_at_one_texel() {
        let mut averages = Vec::new();
        for layer in 0..LAYERS {
            let chain = mip_chain(layer);
            assert_eq!(chain.len(), MIP_COUNT as usize);
            for (width, height, pixels) in &chain {
                assert_eq!(pixels.len(), (width * height * 4) as usize);
                assert!(pixels.as_chunks::<4>().0.iter().all(|p| p[3] == 255));
            }
            let last = chain.last().unwrap();
            assert_eq!((last.0, last.1), (1, 1));
            assert!(!averages.contains(&last.2));
            averages.push(last.2.clone());
        }
    }

    #[test]
    fn texture_field_has_no_longitude_seam_or_pole_special_case() {
        for layer in 0..LAYERS {
            assert_eq!(
                albedo([-1.0, 0.0, 0.0], layer),
                albedo([-1.0, 0.0, -0.0], layer)
            );
            assert_eq!(
                albedo([0.0, 1.0, 0.0], layer),
                albedo([0.0, 1.0, -0.0], layer)
            );
        }
    }
}
