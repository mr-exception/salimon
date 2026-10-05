//! Small, deterministic fragment simulation in ship-local or planet-relative metres.
//! Spherical contacts bound the irregular display meshes without a physics engine.
use std::time::Duration;

use crate::mining::MiningTool;
use salimon_character::{SHIP_FLOOR_HEIGHT_METERS, ShipFrame, ship_floor_placement};
use salimon_world::resources::{FragmentId, ResourceTransform};
use salimon_world::{BodyRole, CELESTIAL_BODIES, WorldPosition};

const GRAVITY: f64 = 9.81;
const RESTITUTION: f64 = 0.12;
const MAX_STEP: f64 = 1.0 / 90.0;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct FragmentMotion {
    pub(crate) velocity: [f64; 3],
}

#[derive(Clone, Copy)]
struct Body {
    center: [f64; 3],
    radius: f64,
}

#[derive(Clone, Copy)]
struct Piece {
    id: FragmentId,
    position: [f64; 3],
    velocity: [f64; 3],
    radius: f64,
    ship: bool,
    body: Option<Body>,
    orientation: [f64; 4],
}

pub(crate) fn release(
    tool: &mut MiningTool,
    id: FragmentId,
    player_forward: [f64; 3],
    frame: ShipFrame,
    inside: bool,
) {
    let velocity = if inside {
        [
            dot(player_forward, frame.axes[0]) * 1.1,
            0.35,
            dot(player_forward, frame.axes[2]) * 1.1,
        ]
    } else {
        let position = tool
            .session
            .fragments()
            .iter()
            .find(|p| p.id() == id)
            .expect("released fragment exists")
            .transform()
            .position()
            .meters();
        let up = nearest_body(position)
            .map(|body| normalize(sub(position, body.center)))
            .unwrap_or([0.0, 1.0, 0.0]);
        add(scale(player_forward, 1.1), scale(up, 0.3))
    };
    tool.fragment_motion.insert(id, FragmentMotion { velocity });
}

pub(crate) fn advance(tool: &mut MiningTool, frame: ShipFrame, delta: Duration, player: [f64; 3]) {
    let elapsed = delta.as_secs_f64();
    if elapsed <= 0.0 {
        return;
    }
    let carried = tool.session.carried_id();
    let mut pieces: Vec<_> = tool
        .session
        .fragments()
        .iter()
        .copied()
        .filter(|p| Some(p.id()) != carried)
        .filter(|p| length(sub(p.transform().position().meters(), player)) < 125.0)
        .map(|p| {
            let ship = tool.ship_fragments.contains_key(&p.id());
            let position = if ship {
                tool.ship_fragments[&p.id()]
            } else {
                p.transform().position().meters()
            };
            let body = if ship { None } else { nearest_body(position) };
            let initial = if tool.fragment_motion.contains_key(&p.id()) {
                tool.fragment_motion[&p.id()].velocity
            } else if let Some(body) = body {
                // New mined pieces spring outward from the source rather than arriving in a row.
                let up = normalize(sub(position, body.center));
                let axis = if up[1].abs() < 0.9 {
                    [0.0, 1.0, 0.0]
                } else {
                    [1.0, 0.0, 0.0]
                };
                let tangent = normalize(cross(up, axis));
                add(
                    scale(up, 1.55),
                    scale(tangent, 0.70 + (p.id().0 % 3) as f64 * 0.12),
                )
            } else {
                [0.0; 3]
            };
            Piece {
                id: p.id(),
                position,
                velocity: initial,
                radius: salimon_world::resource_fragments::side_meters(p) * 0.5,
                ship,
                body,
                orientation: p.transform().orientation_xyzw(),
            }
        })
        .collect();
    if pieces.is_empty() {
        return;
    }
    // A fixed maximum substep prevents fast ejected pieces from tunnelling through a pile.
    let steps = (elapsed / MAX_STEP).ceil().clamp(1.0, 48.0) as usize;
    let dt = elapsed / steps as f64;
    for _ in 0..steps {
        for piece in &mut pieces {
            let previous = piece.position;
            let up = piece
                .body
                .map(|body| normalize(sub(piece.position, body.center)))
                .unwrap_or([0.0, 1.0, 0.0]);
            piece.velocity = sub(piece.velocity, scale(up, GRAVITY * dt));
            piece.position = add(piece.position, scale(piece.velocity, dt));
            if piece.ship && ship_floor_placement(piece.position, piece.radius).is_none() {
                piece.position[0] = previous[0];
                piece.position[2] = previous[2];
                piece.velocity[0] *= -0.15;
                piece.velocity[2] *= -0.15;
            }
            ground_contact(piece);
        }
        for _ in 0..3 {
            for i in 0..pieces.len() {
                for j in i + 1..pieces.len() {
                    let (left, right) = pieces.split_at_mut(j);
                    let a = &mut left[i];
                    let b = &mut right[0];
                    if a.ship != b.ship
                        || (a.body.is_some()
                            && b.body.is_some()
                            && a.body.unwrap().center != b.body.unwrap().center)
                    {
                        continue;
                    }
                    let separation = sub(b.position, a.position);
                    let distance = length(separation);
                    let minimum = a.radius + b.radius;
                    if distance >= minimum {
                        continue;
                    }
                    let normal = if distance > 1e-9 {
                        scale(separation, 1.0 / distance)
                    } else {
                        [0.0, 1.0, 0.0]
                    };
                    let correction = scale(normal, (minimum - distance + 0.0001) * 0.5);
                    let old_a = a.position;
                    let old_b = b.position;
                    a.position = sub(a.position, correction);
                    b.position = add(b.position, correction);
                    if a.ship && ship_floor_placement(a.position, a.radius).is_none() {
                        a.position[0] = old_a[0];
                        a.position[2] = old_a[2];
                    }
                    if b.ship && ship_floor_placement(b.position, b.radius).is_none() {
                        b.position[0] = old_b[0];
                        b.position[2] = old_b[2];
                    }
                    let approach = dot(sub(b.velocity, a.velocity), normal);
                    if approach < 0.0 {
                        let impulse = -(1.0 + RESTITUTION) * approach * 0.5;
                        a.velocity = sub(a.velocity, scale(normal, impulse));
                        b.velocity = add(b.velocity, scale(normal, impulse));
                    }
                    ground_contact(a);
                    ground_contact(b);
                }
            }
        }
        for piece in &mut pieces {
            piece.velocity = scale(piece.velocity, 0.995);
            if length(piece.velocity) < 0.015 {
                piece.velocity = [0.0; 3];
            }
        }
    }
    for piece in pieces {
        tool.fragment_motion.insert(
            piece.id,
            FragmentMotion {
                velocity: piece.velocity,
            },
        );
        let position = if piece.ship {
            tool.ship_fragments.insert(piece.id, piece.position);
            frame.local_to_world(piece.position)
        } else {
            piece.position
        };
        let pose = ResourceTransform::new(
            WorldPosition::new(position[0], position[1], position[2]),
            piece.orientation,
        )
        .expect("finite fragment physics pose");
        tool.session.move_loose(piece.id, pose);
    }
}

fn ground_contact(piece: &mut Piece) {
    let (up, penetration) = if piece.ship {
        (
            [0.0, 1.0, 0.0],
            SHIP_FLOOR_HEIGHT_METERS + piece.radius - piece.position[1],
        )
    } else if let Some(body) = piece.body {
        let radial = sub(piece.position, body.center);
        let distance = length(radial);
        (normalize(radial), body.radius + piece.radius - distance)
    } else {
        return;
    };
    if penetration <= 0.0 {
        return;
    }
    piece.position = add(piece.position, scale(up, penetration));
    let downward = dot(piece.velocity, up);
    if downward < 0.0 {
        piece.velocity = sub(piece.velocity, scale(up, downward * (1.0 + RESTITUTION)));
    }
    let normal_speed = dot(piece.velocity, up);
    piece.velocity = add(
        scale(sub(piece.velocity, scale(up, normal_speed)), 0.88),
        scale(up, normal_speed),
    );
}

fn nearest_body(position: [f64; 3]) -> Option<Body> {
    CELESTIAL_BODIES
        .iter()
        .filter(|body| body.role == BodyRole::Solid)
        .min_by(|a, b| {
            (length(sub(position, a.center.meters())) - a.radius_meters)
                .abs()
                .total_cmp(&(length(sub(position, b.center.meters())) - b.radius_meters).abs())
        })
        .map(|body| Body {
            center: body.center.meters(),
            radius: body.radius_meters,
        })
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}
fn length(a: [f64; 3]) -> f64 {
    dot(a, a).sqrt()
}
fn normalize(a: [f64; 3]) -> [f64; 3] {
    let len = length(a);
    if len > 1e-12 {
        scale(a, 1.0 / len)
    } else {
        [0.0, 1.0, 0.0]
    }
}
fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|i| a[i] + b[i])
}
fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}
fn scale(a: [f64; 3], s: f64) -> [f64; 3] {
    a.map(|v| v * s)
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use salimon_world::CelestialBodyId;
    use salimon_world::resources::{DepositId, RawMaterial, ResourceDeposit, ResourceId};

    fn frame() -> ShipFrame {
        ShipFrame {
            origin_meters: [0.0; 3],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        }
    }
    fn mined(mass: f64) -> MiningTool {
        let mut tool = MiningTool::default();
        let mut deposit = ResourceDeposit::new(
            DepositId {
                body: CelestialBodyId::Earth,
                local: 1,
            },
            RawMaterial::new(ResourceId::SilicateRock, mass).unwrap(),
            WorldPosition::new(0.0, 0.0, 0.0),
            mass,
        )
        .unwrap();
        tool.session
            .extract(&mut deposit, Duration::from_secs_f64(mass / 2.0));
        tool
    }

    #[test]
    fn released_fragment_falls_from_hand_and_settles_on_deck() {
        let mut tool = mined(2.0);
        let id = tool.session.fragments()[0].id();
        tool.ship_fragments.insert(id, [-5.0, 1.3, 0.0]);
        tool.fragment_motion.insert(id, FragmentMotion::default());
        release(&mut tool, id, [1.0, 0.0, 0.0], frame(), true);
        advance(
            &mut tool,
            frame(),
            Duration::from_millis(100),
            [-5.0, 2.0, 0.0],
        );
        let falling = tool.ship_fragments[&id];
        assert!(falling[0] > -5.0 && falling[1] < 1.3);
        for _ in 0..180 {
            advance(
                &mut tool,
                frame(),
                Duration::from_millis(16),
                [-5.0, 2.0, 0.0],
            );
        }
        let settled = tool.ship_fragments[&id];
        let radius =
            salimon_world::resource_fragments::side_meters(tool.session.fragments()[0]) * 0.5;
        assert!((settled[1] - SHIP_FLOOR_HEIGHT_METERS - radius).abs() < 0.002);
    }

    #[test]
    fn two_fragments_contact_and_stack_on_the_deck() {
        let mut tool = mined(4.0);
        let pieces = tool.session.fragments().to_vec();
        for (i, piece) in pieces.iter().enumerate() {
            tool.ship_fragments
                .insert(piece.id(), [-5.0, 0.5 + i as f64 * 0.4, 0.0]);
            tool.fragment_motion
                .insert(piece.id(), FragmentMotion::default());
        }
        for _ in 0..150 {
            advance(
                &mut tool,
                frame(),
                Duration::from_millis(16),
                [-5.0, 2.0, 0.0],
            );
        }
        let a = tool.ship_fragments[&pieces[0].id()];
        let b = tool.ship_fragments[&pieces[1].id()];
        let radius = salimon_world::resource_fragments::side_meters(pieces[0]) * 0.5;
        assert!(a[1].min(b[1]) >= SHIP_FLOOR_HEIGHT_METERS + radius - 0.002);
        assert!(length(sub(a, b)) >= radius * 2.0 - 0.002);
        assert!(a[1].max(b[1]) > SHIP_FLOOR_HEIGHT_METERS + radius * 2.0);
    }

    #[test]
    fn freshly_mined_fragment_ejects_then_returns_to_surface() {
        let body = CELESTIAL_BODIES
            .iter()
            .find(|body| body.id == CelestialBodyId::Earth)
            .unwrap();
        let mut deposit = ResourceDeposit::new(
            DepositId {
                body: body.id,
                local: 9,
            },
            RawMaterial::new(ResourceId::IronOre, 2.0).unwrap(),
            body.center.translated([0.0, body.radius_meters, 0.0]),
            2.0,
        )
        .unwrap();
        let mut tool = MiningTool::default();
        tool.session.extract(&mut deposit, Duration::from_secs(1));
        let before = tool.session.fragments()[0].transform().position().meters();
        let player = deposit.position().translated([0.0, 1.7, 0.0]).meters();
        advance(&mut tool, frame(), Duration::from_millis(50), player);
        let popped = tool.session.fragments()[0].transform().position().meters();
        assert!(length(sub(popped, before)) > 0.03);
        assert!(popped[1] > before[1]);
        for _ in 0..120 {
            advance(&mut tool, frame(), Duration::from_millis(16), player);
        }
        let settled = tool.session.fragments()[0].transform().position().meters();
        let radius =
            salimon_world::resource_fragments::side_meters(tool.session.fragments()[0]) * 0.5;
        assert!(
            (length(sub(settled, body.center.meters())) - body.radius_meters - radius).abs()
                < 0.002
        );
    }
}
