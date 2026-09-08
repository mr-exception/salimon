use super::*;
use crate::TimeScale;
fn id(s: &str) -> CatalogId {
    CatalogId::from_source("test", s).unwrap()
}
fn epoch() -> Epoch {
    Epoch::calendar([2000, 1, 1], [12, 0, 0, 0], TimeScale::Tt).unwrap()
}
fn state() -> State {
    State {
        meters: [0.125, -0.25, 0.5],
        meters_per_second: [0.5, 0.25, -0.125],
    }
}
fn tree() -> FrameTree {
    FrameTree::new(
        SystemAnchor {
            system: id("s"),
            epoch: epoch(),
            position: GalacticPosition::new([860_000, -92_000, 71_000], [0.0; 3]).unwrap(),
            velocity_mps: [3e4, -2e4, 1e4],
            rotation: Rotation::IDENTITY,
        },
        vec![
            FrameLink {
                child: FrameId::Body(id("b")),
                parent: FrameId::System(id("s")),
                epoch: epoch(),
                origin_meters: [1.5e11, 2e10, -3e10],
                velocity_mps: [3e4, 0.0, 0.0],
                rotation: Rotation::new([[0.0, 1.0, 0.0], [-1.0, 0.0, 0.0], [0.0, 0.0, 1.0]])
                    .unwrap(),
                angular_velocity_rps: [0.0, 0.0, 1e-5],
            },
            FrameLink {
                child: FrameId::Local(id("deck")),
                parent: FrameId::Body(id("b")),
                epoch: epoch(),
                origin_meters: [10.0, 0.0, 0.0],
                velocity_mps: [1.0, 2.0, 0.0],
                rotation: Rotation::IDENTITY,
                angular_velocity_rps: [0.0, 0.0, 0.25],
            },
        ],
    )
    .unwrap()
}
fn near(a: [f64; 3], b: [f64; 3], e: f64) {
    for i in 0..3 {
        assert!((a[i] - b[i]).abs() <= e, "{a:?} != {b:?}");
    }
}
#[test]
fn submeter_details_survive_million_cell_anchor_and_gpu_subtraction() {
    let p = GalacticPosition::new([860_000, -92_000, 71_000], [12.0, -3.0, 9.0]).unwrap();
    let q = p.translated([0.125, -0.25, 0.5]).unwrap();
    assert_eq!(q.relative_meters(p).unwrap(), state().meters);
    assert_eq!(q.camera_relative_f32(p).unwrap(), [0.125, -0.25, 0.5]);
    // Opposite cell boundary retains submeter difference without global f64.
    let a = GalacticPosition::new([100_000; 3], [CELL_METERS / 2.0 - 0.125; 3]).unwrap();
    let b = a.translated([0.25; 3]).unwrap();
    assert_eq!(b.relative_meters(a).unwrap(), [0.25; 3]);
    assert_eq!(b.cells(), [100_001; 3]);
}
#[test]
fn nested_moving_rotating_frames_roundtrip_and_common_ancestor_precision() {
    let t = tree();
    let local = FrameId::Local(id("deck"));
    let body = FrameId::Body(id("b"));
    let system = FrameId::System(id("s"));
    let b = t.transform(&local, &body, state(), epoch()).unwrap();
    near(b.meters, [10.125, -0.25, 0.5], 0.0);
    near(b.meters_per_second, [1.5625, 2.28125, -0.125], 0.0);
    let s = t.transform(&local, &system, state(), epoch()).unwrap();
    let back = t.transform(&system, &local, s, epoch()).unwrap();
    near(back.meters, state().meters, 1e-5);
    near(back.meters_per_second, state().meters_per_second, 1e-9);
    let galactic = t.galactic_position(&local, state(), epoch()).unwrap();
    let velocity = add(t.anchor.velocity_mps, s.meters_per_second);
    let back = t
        .from_galactic(&local, galactic, velocity, epoch())
        .unwrap();
    near(back.meters, state().meters, 0.001);
    near(back.meters_per_second, state().meters_per_second, 1e-8);
}
#[test]
fn proper_and_radial_motion_retain_local_offset() {
    let t = tree();
    let later = epoch().advance_nanos(2_000_000_000).unwrap();
    let moved = t.anchor.position_at(later).unwrap();
    assert_eq!(
        moved.relative_meters(t.anchor.position).unwrap(),
        [60000.0, -40000.0, 20000.0]
    );
}
#[test]
fn invalid_frames_epochs_rotations_and_positions_reject() {
    let t = tree();
    let frame = FrameId::Body(id("b"));
    assert!(
        t.transform(&frame, &frame, state(), epoch().advance_nanos(1).unwrap())
            .is_err()
    );
    assert!(
        t.transform(&FrameId::System(id("wrong")), &frame, state(), epoch())
            .is_err()
    );
    let mut links = t.links.values().cloned().collect::<Vec<_>>();
    links[0].parent = links[0].child.clone();
    assert!(FrameTree::new(t.anchor.clone(), links).is_err());
    assert!(Rotation::new([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, -1.0]]).is_err());
    assert!(GalacticPosition::new([0; 3], [f64::NAN; 3]).is_err());
    assert!(GalacticPosition::new([i64::MAX; 3], [CELL_METERS; 3]).is_err());
    assert!(
        serde_json::from_str::<GalacticPosition>(r#"{"cell":[0,0,0],"offset":[0,0,0],"extra":1}"#)
            .is_err()
    );
}
#[test]
fn gregorian_epoch_rollovers_scale_mismatch_and_wire_validation() {
    let e = Epoch::calendar([2000, 2, 28], [23, 59, 59, 999_999_999], TimeScale::Tt).unwrap();
    assert_eq!(
        e.advance_nanos(1).unwrap(),
        Epoch::calendar([2000, 2, 29], [0; 4], TimeScale::Tt).unwrap()
    );
    let next = Epoch::calendar([2000, 3, 1], [0; 4], TimeScale::Tt).unwrap();
    assert_eq!(next.seconds_since(e).unwrap(), 86400.000000001);
    assert!(Epoch::calendar([1900, 2, 29], [0; 4], TimeScale::Tt).is_err());
    assert!(Epoch::calendar([2000, 1, 1], [0, 0, 60, 0], TimeScale::Utc).is_err());
    assert!(
        epoch()
            .seconds_since(Epoch::calendar([2000, 1, 1], [12, 0, 0, 0], TimeScale::Tdb).unwrap())
            .is_err()
    );
    let utc = Epoch::calendar([2000, 1, 1], [0; 4], TimeScale::Utc).unwrap();
    assert!(utc.advance_nanos(-1).is_err());
    assert_eq!(
        utc.advance_nanos(1).unwrap().seconds_since(utc).unwrap(),
        1e-9
    );
    assert!(
        serde_json::from_str::<Epoch>(r#"{"day":0,"nanos":86400000000000,"scale":"tt"}"#).is_err()
    );
    assert_eq!(
        serde_json::from_slice::<Epoch>(&serde_json::to_vec(&epoch()).unwrap()).unwrap(),
        epoch()
    );
}
