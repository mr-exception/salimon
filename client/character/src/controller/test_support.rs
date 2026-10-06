//! Shared regression fixtures; absent from production builds.
use super::*;
pub(super) fn frame() -> ShipFrame {
    ShipFrame {
        origin_meters: [0.0, 10.0, 0.0],
        axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
    }
}

pub(super) fn surface() -> SurfaceFrame {
    SurfaceFrame {
        body_center_meters: [0.0, -90.0, 0.0],
        radius_meters: 100.0,
    }
}

pub(super) fn inside_at(x: f64, z: f64) -> CharacterController {
    CharacterController {
        position: PositionState::Inside {
            local: [x, PLAYER_START[1], z],
        },
        ..CharacterController::default()
    }
}

pub(super) fn walk_steps(controller: &mut CharacterController, input: MovementInput, steps: u32) {
    for _ in 0..steps {
        controller.advance(
            Duration::from_millis(100),
            input,
            frame(),
            surface(),
            false,
            true,
        );
    }
}

pub(super) fn rotated_catalog_frames() -> (ShipFrame, SurfaceFrame) {
    let ship = ShipFrame {
        origin_meters: [1.0e12, -750.0e9, 250.0e9],
        axes: [
            [
                0.904_961_439_349_810_7,
                0.408_776_116_798_125_6,
                -0.118_096_907_772_236_55,
            ],
            [
                0.425_493_587_836_431_4,
                -0.869_405_870_275_219_2,
                0.251_174_519_888_204_05,
            ],
            [0.0, -0.277_552_732_046_423_83, -0.960_710_404_301_715_7],
        ],
    };
    let surface = SurfaceFrame {
        body_center_meters: sub(ship.origin_meters, scale(ship.axes[1], 6_000_000.0)),
        radius_meters: 6_000_000.0,
    };
    (ship, surface)
}
