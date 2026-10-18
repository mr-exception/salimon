//! Narrow position DTOs for world/ship conversion boundaries, not vector math.

/// Absolute f64 meters. Construction assigns a frame; it does not validate or normalize.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldPoint([f64; 3]);

impl WorldPoint {
    #[must_use]
    pub const fn new(meters: [f64; 3]) -> Self {
        Self(meters)
    }
    #[must_use]
    pub const fn meters(self) -> [f64; 3] {
        self.0
    }
}

/// Ship-local f64 meters: +X forward, +Y up, +Z port.
/// No implicit conversion from absolute positions or raw arrays is provided.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShipLocalPoint([f64; 3]);

impl ShipLocalPoint {
    #[must_use]
    pub const fn new(meters: [f64; 3]) -> Self {
        Self(meters)
    }
    #[must_use]
    pub const fn meters(self) -> [f64; 3] {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ShipFrame;

    #[test]
    fn typed_ship_round_trip_retains_distant_anchor_and_handedness() {
        // Exact quarter-turn axes plus an oblique orthonormal frame exercise
        // both transform orders. 0.5 mm covers f64 ULPs near 1e12 m.
        let angle = 0.73_f64;
        for axes in [
            [[0.0, 0.0, 1.0], [0.0, 1.0, 0.0], [-1.0, 0.0, 0.0]],
            [
                [angle.cos(), 0.0, angle.sin()],
                [0.0, 1.0, 0.0],
                [-angle.sin(), 0.0, angle.cos()],
            ],
        ] {
            for origin in [[0.0; 3], [1e12, -7.5e11, 2.5e11]] {
                let frame = ShipFrame {
                    origin_meters: origin,
                    axes,
                };
                for meters in [[0.125, -0.25, 1.0], [-5.8, 1.3, 2.7]] {
                    let local = ShipLocalPoint::new(meters);
                    let world = frame.unproject_local(local);
                    let round_trip = frame.project_world(world);
                    for (actual, expected) in round_trip.meters().into_iter().zip(meters) {
                        assert!((actual - expected).abs() < 0.0005);
                    }
                    for (actual, expected) in frame
                        .unproject_local(round_trip)
                        .meters()
                        .into_iter()
                        .zip(world.meters())
                    {
                        assert!((actual - expected).abs() < 0.0005);
                    }
                    assert_eq!(world.meters(), frame.local_to_world(meters));
                }
                assert_eq!(salimon_math::cross(axes[0], axes[1]), axes[2]);
            }
        }
    }
}
