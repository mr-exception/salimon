use super::*;
use crate::structure::{MemberShape, StructuralMass, StructuralPart, StructureId};
fn structure() -> ShipStructure {
    ShipStructure::new(
        [0.; 3],
        StructuralPart {
            id: StructureId(1),
            cells: vec![Cell::new(0, 0, 0)],
            shape: MemberShape::Frame,
            faces: vec![],
            space: Envelope {
                min: [0.; 3],
                max: [3.; 3],
            },
            collision: vec![],
            mass: StructuralMass::IncludedInDryHull,
        },
    )
    .unwrap()
}
fn catalog() -> Catalog {
    Catalog {
        schema: 1,
        approved_economy: false,
        blueprints: vec![Blueprint {
            id: "stool".into(),
            name: "Stool".into(),
            category: Category::Furniture,
            snap: Snap::Fine,
            fine_pitch_meters: 0.5,
            footprint: Footprint::Interior,
            cells: vec![Cell::new(0, 0, 0)],
            space: Envelope {
                min: [0.; 3],
                max: [0.5, 1., 0.5],
            },
            requirements: vec![Requirement {
                resource: salimon_world::resources::ResourceId::IronOre,
                milligrams: 1_000_000,
            }],
            duration_seconds: 10,
            hull_affecting: false,
            flight_eligible: true,
        }],
    }
}
fn context(obstacles: &[Obstacle]) -> PlacementContext<'_> {
    PlacementContext {
        actor: 1,
        owner: 1,
        flight: FlightPermission::Landed,
        eye_meters: [1.5, 1.8, 1.5],
        reach_meters: 5.,
        reachable: true,
        physical_support: true,
        obstacles,
    }
}
#[test]
fn quarter_turns_transform_half_open_cells_without_drift_or_wrapping() {
    assert_eq!(
        Rotation::Port.cell(Cell::new(0, 1, 0)),
        Some(Cell::new(-1, 1, 0))
    );
    for rotation in [
        Rotation::Forward,
        Rotation::Port,
        Rotation::Aft,
        Rotation::Starboard,
    ] {
        let p = Placements::default()
            .preview(
                &catalog(),
                "stool",
                [1.9, 0., 1.9],
                rotation,
                &structure(),
                &context(&[]),
            )
            .unwrap();
        assert_eq!(p.anchor_meters, [1.5, 0., 1.5]);
        assert!(p.rejection.is_none());
        assert!((p.space.max[0] - p.space.min[0] - 0.5).abs() < 1e-12);
    }
    assert_eq!(
        Rotation::Port.cell(Cell::new(0, 0, i32::MIN)),
        Some(Cell::new(i32::MAX, 0, 0))
    );
}
#[test]
fn reservations_overlap_atomically_and_cancellation_does_not_reuse_identity() {
    let mut sites = Placements::default();
    let s = structure();
    let c = context(&[]);
    let cat = catalog();
    let first = sites
        .place(&cat, "stool", [1., 0., 1.], Rotation::Forward, &s, &c)
        .unwrap();
    let before = sites.clone();
    assert_eq!(
        sites.place(&cat, "stool", [1., 0., 1.], Rotation::Forward, &s, &c),
        Err(PlacementError::Overlapping(first))
    );
    assert_eq!(sites, before);
    assert_eq!(sites.cancel(first, 2), Err(PlacementError::Unauthorized));
    assert_eq!(sites, before);
    sites.cancel(first, 1).unwrap();
    assert_eq!(
        sites.cancel(first, 1),
        Err(PlacementError::MissingPlaceholder)
    );
    let next = sites
        .place(&cat, "stool", [1., 0., 1.], Rotation::Forward, &s, &c)
        .unwrap();
    assert!(next.0 > first.0);
}
#[test]
fn player_cargo_installed_geometry_and_escape_routes_block_preview_and_final_placement() {
    for kind in [
        ObstacleKind::Player,
        ObstacleKind::Cargo,
        ObstacleKind::Installed,
        ObstacleKind::ProtectedRoute,
    ] {
        let blocked = [Obstacle {
            kind,
            space: Envelope {
                min: [1., 0., 1.],
                max: [2.; 3],
            },
        }];
        let mut sites = Placements::default();
        let s = structure();
        let cat = catalog();
        let c = context(&blocked);
        assert_eq!(
            sites
                .preview(&cat, "stool", [1., 0., 1.], Rotation::Forward, &s, &c)
                .unwrap()
                .rejection,
            Some(PlacementError::Blocked(kind))
        );
        assert_eq!(
            sites.place(&cat, "stool", [1., 0., 1.], Rotation::Forward, &s, &c),
            Err(PlacementError::Blocked(kind))
        );
        assert_eq!(sites.jobs().count(), 0);
    }
}
#[test]
fn completion_revalidates_new_cargo_support_and_flight_without_a_player_reach_requirement() {
    let mut sites = Placements::default();
    let s = structure();
    let cat = catalog();
    let id = sites
        .place(
            &cat,
            "stool",
            [1., 0., 1.],
            Rotation::Forward,
            &s,
            &context(&[]),
        )
        .unwrap();
    let cargo = [Obstacle {
        kind: ObstacleKind::Cargo,
        space: Envelope {
            min: [1., 0., 1.],
            max: [2.; 3],
        },
    }];
    assert_eq!(
        sites.validate_completion(id, &s, &context(&cargo)),
        Err(PlacementError::Blocked(ObstacleKind::Cargo))
    );
    let mut c = context(&[]);
    c.eye_meters = [1000.; 3];
    assert!(sites.validate_completion(id, &s, &c).is_ok());
    c.physical_support = false;
    assert_eq!(
        sites.validate_completion(id, &s, &c),
        Err(PlacementError::Unsupported)
    );
}
#[test]
fn permission_reach_and_unsupported_geometry_fail_without_publishing_projects() {
    let s = structure();
    let cat = catalog();
    let sites = Placements::default();
    let mut c = context(&[]);
    c.actor = 2;
    assert_eq!(
        sites
            .preview(&cat, "stool", [1., 0., 1.], Rotation::Forward, &s, &c)
            .unwrap()
            .rejection,
        Some(PlacementError::Unauthorized)
    );
    c = context(&[]);
    c.eye_meters = [100.; 3];
    assert_eq!(
        sites
            .preview(&cat, "stool", [1., 0., 1.], Rotation::Forward, &s, &c)
            .unwrap()
            .rejection,
        Some(PlacementError::OutOfReach)
    );
    c = context(&[]);
    c.reachable = false;
    assert_eq!(
        sites
            .preview(&cat, "stool", [1., 0., 1.], Rotation::Forward, &s, &c)
            .unwrap()
            .rejection,
        Some(PlacementError::Unreachable)
    );
    assert_eq!(
        sites
            .preview(
                &cat,
                "stool",
                [4., 0., 1.],
                Rotation::Forward,
                &s,
                &context(&[])
            )
            .unwrap()
            .rejection,
        Some(PlacementError::Unsupported)
    );
}
#[test]
fn one_multideck_room_reserves_many_cells_and_never_uses_an_unbuilt_neighbor_as_support() {
    let mut cat = catalog();
    let b = &mut cat.blueprints[0];
    b.id = "room".into();
    b.category = Category::Room;
    b.snap = Snap::Structural;
    b.footprint = Footprint::Structural;
    b.hull_affecting = true;
    b.flight_eligible = false;
    b.space.max = [3., 6., 3.];
    b.cells.push(Cell::new(0, 1, 0));
    let s = structure();
    let mut c = context(&[]);
    c.eye_meters = [3., 2., 1.];
    let mut sites = Placements::default();
    let id = sites
        .place(&cat, "room", [3.9, 0., 0.], Rotation::Forward, &s, &c)
        .unwrap();
    let job = sites.inspect(id).unwrap();
    assert_eq!(job.cells.len(), 2);
    assert_eq!(job.anchor_meters, [3., 0., 0.]);
    assert_eq!(sites.jobs().count(), 1);
    c.flight = FlightPermission::Airborne;
    assert_eq!(
        sites.validate_completion(id, &s, &c),
        Err(PlacementError::FlightRestricted)
    );
    c.flight = FlightPermission::Landed;
    assert_eq!(
        sites
            .preview(&cat, "room", [6., 0., 0.], Rotation::Forward, &s, &c)
            .unwrap()
            .rejection,
        Some(PlacementError::Unsupported)
    );
}
#[test]
fn consequences_restrict_airborne_modules_even_if_the_category_is_interior() {
    let mut cat = catalog();
    let b = &mut cat.blueprints[0];
    b.category = Category::Module;
    b.hull_affecting = true;
    b.flight_eligible = false;
    let mut c = context(&[]);
    c.flight = FlightPermission::Airborne;
    assert_eq!(
        Placements::default()
            .preview(
                &cat,
                "stool",
                [1., 0., 1.],
                Rotation::Forward,
                &structure(),
                &c
            )
            .unwrap()
            .rejection,
        Some(PlacementError::FlightRestricted)
    );
}
#[test]
fn malformed_catalog_geometry_and_tampered_saved_cells_are_rejected() {
    let mut cat = catalog();
    cat.blueprints[0].space.max[0] = 1e30;
    assert!(cat.validate().is_err());
    let cat = catalog();
    assert!(
        Placements::default()
            .preview(
                &cat,
                "stool",
                [f64::NAN, 0., 0.],
                Rotation::Forward,
                &structure(),
                &context(&[])
            )
            .is_err()
    );
    let mut sites = Placements::default();
    let id = sites
        .place(
            &cat,
            "stool",
            [1., 0., 1.],
            Rotation::Forward,
            &structure(),
            &context(&[]),
        )
        .unwrap();
    sites.validate_save().unwrap();
    let bytes = serde_json::to_vec(&sites).unwrap();
    let restored: Placements = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(sites, restored);
    sites.jobs.get_mut(&id).unwrap().cells = vec![Cell::new(100, 0, 0)];
    assert_eq!(sites.validate_save(), Err(PlacementError::InvalidGeometry));
}

#[test]
fn duplicate_saved_project_id_is_rejected_before_publication() {
    let mut sites = Placements::default();
    sites
        .place(
            &catalog(),
            "stool",
            [1., 0., 1.],
            Rotation::Forward,
            &structure(),
            &context(&[]),
        )
        .unwrap();
    let mut source = serde_json::to_value(&sites).unwrap();
    let duplicate = source["jobs"][0].clone();
    source["jobs"].as_array_mut().unwrap().push(duplicate);
    assert!(serde_json::from_value::<Placements>(source).is_err());
}
