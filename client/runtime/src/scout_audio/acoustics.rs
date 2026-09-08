//! Cheap geometric partition separation. Observation only; never pressure/physics.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Partition {
    pub axis: usize,
    pub at: f64,
    pub span: [(f64, f64); 3],
    pub openness: f64,
    pub closed_gain: f64,
    pub closed_filter: f64,
}
/// A segment crosses each physical boundary at most once. Finite apertures permit
/// sleep screens; an unbounded plane represents a connected cargo room/door.
pub(crate) fn transmission(
    eye: [f64; 3],
    source: [f64; 3],
    partitions: &[Partition],
) -> (f64, f64) {
    let mut gain = 1.;
    let mut filter = 1.;
    for p in partitions.iter().take(8) {
        let a = eye[p.axis] - p.at;
        let b = source[p.axis] - p.at;
        if a * b >= 0. || (a - b).abs() < 1e-9 {
            continue;
        }
        let t = a / (a - b);
        let hit = std::array::from_fn::<_, 3, _>(|i| eye[i] + (source[i] - eye[i]) * t);
        if (0..3).any(|i| i != p.axis && (hit[i] < p.span[i].0 || hit[i] > p.span[i].1)) {
            continue;
        }
        let open = p.openness.clamp(0., 1.);
        gain *= p.closed_gain + (1. - p.closed_gain) * open;
        filter *= p.closed_filter + (1. - p.closed_filter) * open;
    }
    (gain, filter)
}
pub(crate) fn scout(door: f64, screen: f64) -> [Partition; 2] {
    [
        Partition {
            axis: 0,
            at: salimon_character::cargo::CARGO_DOOR_TARGET[0],
            span: [(f64::NEG_INFINITY, f64::INFINITY); 3],
            openness: door,
            closed_gain: 0.22,
            closed_filter: 0.08,
        },
        Partition {
            axis: 2,
            at: -2.92,
            span: [(-1.87, 0.27), (0.575, 2.025), (0., 0.)],
            openness: screen,
            closed_gain: 0.72,
            closed_filter: 0.4,
        },
    ]
}
pub(crate) fn area(p: [f64; 3]) -> &'static str {
    if p[0] < salimon_character::cargo::CARGO_DOOR_TARGET[0] {
        "cargo"
    } else if p[0] >= salimon_character::COCKPIT_SEAT_MARKER_METERS[0] - 1.5 {
        "cockpit"
    } else {
        "habitation/engineering"
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn physical_rooms_apertures_distance_and_multiple_boundaries() {
        let open = scout(1., 1.);
        let shut = scout(0., 0.);
        assert_eq!(transmission([-6., 1., 0.], [0., 1., 0.], &open), (1., 1.));
        assert_eq!(
            transmission([-6., 1., 0.], [0., 1., 0.], &shut),
            (0.22, 0.08)
        );
        assert_eq!(
            transmission([0., 1., -3.5], [0., 1., 0.], &shut),
            (0.72, 0.4)
        );
        assert_eq!(transmission([2., 1., -3.5], [2., 1., 0.], &shut), (1., 1.));
        let mut many = shut.to_vec();
        many.push(Partition {
            axis: 0,
            at: -2.,
            ..shut[0]
        });
        let (g, f) = transmission([-6., 1., 0.], [0., 1., 0.], &many);
        assert!((g - 0.0484).abs() < 1e-12);
        assert!((f - 0.0064).abs() < 1e-12);
    }
}
