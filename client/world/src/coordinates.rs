//! Absolute meter coordinates and subtract-before-cast conversion.

/// A canonical world-space position in meters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldPosition {
    meters: [f64; 3],
}

impl WorldPosition {
    #[must_use]
    pub const fn new(x_meters: f64, y_meters: f64, z_meters: f64) -> Self {
        Self {
            meters: [x_meters, y_meters, z_meters],
        }
    }

    #[must_use]
    pub const fn meters(self) -> [f64; 3] {
        self.meters
    }

    #[must_use]
    pub const fn translated(self, offset_meters: [f64; 3]) -> Self {
        Self::new(
            self.meters[0] + offset_meters[0],
            self.meters[1] + offset_meters[1],
            self.meters[2] + offset_meters[2],
        )
    }

    /// Subtracts an origin while both values still have double precision.
    #[must_use]
    pub fn offset_from(self, origin: Self) -> [f64; 3] {
        [
            self.meters[0] - origin.meters[0],
            self.meters[1] - origin.meters[1],
            self.meters[2] - origin.meters[2],
        ]
    }

    /// Euclidean center-to-center distance in authoritative `f64` meters.
    #[must_use]
    pub fn distance_to(self, other: Self) -> f64 {
        let offset = self.offset_from(other);
        offset
            .iter()
            .map(|component| component * component)
            .sum::<f64>()
            .sqrt()
    }

    /// Produces GPU-friendly meters relative to the current camera origin.
    ///
    /// The order is intentional: subtracting global positions after converting
    /// them to `f32` destroys meter-scale offsets at this prototype's anchor.
    #[must_use]
    pub fn camera_relative_f32(self, camera_position: Self) -> [f32; 3] {
        let relative = self.offset_from(camera_position);
        [relative[0] as f32, relative[1] as f32, relative[2] as f32]
    }

    #[must_use]
    pub fn is_finite(self) -> bool {
        self.meters.iter().all(|component| component.is_finite())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subtracting_in_f64_preserves_local_offsets_at_the_large_anchor() {
        let origin = WorldPosition::new(1_000_000_000_000.0, 0.0, 0.0);
        let nearby = origin.translated([1.0, 0.125, -4.0]);

        assert_eq!(nearby.camera_relative_f32(origin), [1.0, 0.125, -4.0]);

        let cast_then_subtract = nearby.meters()[0] as f32 - origin.meters()[0] as f32;
        assert_eq!(cast_then_subtract, 0.0);
    }
}
