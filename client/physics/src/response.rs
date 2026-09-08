//! Contact sampling, warm impulses and exact iterative ground/pair response.
use super::{CONTACT_BIAS_METERS, ObjectState, RESTITUTION, ROLLING_RESISTANCE, Surface, convex};
use salimon_math::{add, cross, dot, length, scale, sub};

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ContactResponse {
    pub(super) pending_warm_start: bool,
    pub(super) transport_response: bool,
    pub(super) normal: [f64; 3],
    pub(super) arms: Vec<([f64; 3], [f64; 3])>,
    pub(super) impulses: Vec<f64>,
    pub(super) friction: Vec<[f64; 3]>,
    pub(super) rolling: [f64; 3],
    pub(super) targets: Vec<f64>,
}
impl ContactResponse {
    pub(super) fn rearm(&mut self, ratio: f64) {
        for impulse in &mut self.impulses {
            *impulse *= ratio;
        }
        for impulse in &mut self.friction {
            *impulse = scale(*impulse, ratio);
        }
        self.rolling = scale(self.rolling, ratio);
        self.targets.fill(0.0);
        self.pending_warm_start = true;
    }
    pub(super) fn prepare(
        &mut self,
        normal: [f64; 3],
        arms: &[([f64; 3], [f64; 3])],
        targets: impl Fn(usize) -> f64,
    ) {
        let transport = self.pending_warm_start || self.transport_response;
        let max_arm_change = if transport { 0.001 } else { 0.0001 };
        let normal_matches = if transport {
            dot(self.normal, normal) > 0.999
        } else {
            length(sub(self.normal, normal)) < 1e-8
        };
        let unchanged = normal_matches
            && self.arms.len() == arms.len()
            && self.arms.iter().zip(arms).all(|(old, new)| {
                length(sub(old.0, new.0)) < max_arm_change
                    && length(sub(old.1, new.1)) < max_arm_change
            });
        if unchanged && transport {
            let normal_factor = dot(self.normal, normal).max(0.0);
            for i in 0..self.impulses.len() {
                self.impulses[i] *= normal_factor;
                self.friction[i] = sub(
                    self.friction[i],
                    scale(normal, dot(self.friction[i], normal)),
                );
                let magnitude = length(self.friction[i]);
                let limit = self.impulses[i] * 0.65;
                if magnitude > limit {
                    self.friction[i] = scale(self.friction[i], limit / magnitude);
                }
            }
            self.normal = normal;
            self.arms.clear();
            self.arms.extend_from_slice(arms);
        }
        self.transport_response = false;
        if !unchanged {
            self.pending_warm_start = false;
            self.normal = normal;
            self.arms = arms.to_vec();
            self.impulses = vec![0.0; arms.len()];
            self.friction = vec![[0.0; 3]; arms.len()];
            self.rolling = [0.0; 3];
            self.targets = (0..arms.len()).map(targets).collect();
        }
    }
}

// A deterministic impulse iteration is redundant only after ALL its mutable
// inputs are bit-identical. No tolerance, iteration-budget or contact change.
#[derive(PartialEq, Eq)]
struct ImpulseIterationState<const N: usize> {
    motion: [[u64; 3]; N],
    normal: u64,
    friction: [u64; 3],
    rolling: [u64; 3],
}
fn iteration_state<const N: usize>(
    motion: [[f64; 3]; N],
    impulses: &[f64],
    friction: &[[f64; 3]],
    rolling: [f64; 3],
) -> Option<ImpulseIterationState<N>> {
    (impulses.len() == 1).then(|| ImpulseIterationState {
        motion: motion.map(|v| v.map(f64::to_bits)),
        normal: impulses[0].to_bits(),
        friction: friction[0].map(f64::to_bits),
        rolling: rolling.map(f64::to_bits),
    })
}

// A bounded contact torque models energy lost to rough-contact rolling/spin.
// Its limit is 10% of normal impulse times bounding radius; no load means no drag.
pub(super) fn rolling_delta(
    relative: [f64; 3],
    previous: &mut [f64; 3],
    limit: f64,
    inverse_angular_mass: f64,
) -> [f64; 3] {
    let mut candidate = sub(*previous, scale(relative, 1.0 / inverse_angular_mass));
    let magnitude = length(candidate);
    if magnitude > limit {
        candidate = scale(candidate, limit / magnitude);
    }
    let delta = sub(candidate, *previous);
    *previous = candidate;
    delta
}

pub(super) fn weightless_ceiling_contact(piece: &mut ObjectState) {
    if let Surface::UnpoweredFloor { ceiling_meters, .. } = piece.surface {
        // Conservative bounding support contains rotating cargo at the ceiling.
        let maximum_center = ceiling_meters - piece.radius;
        if piece.position[1] > maximum_center {
            piece.position[1] = maximum_center;
            if piece.velocity[1] > 0.0 {
                piece.velocity[1] *= -RESTITUTION;
            }
        }
    }
}

#[cfg(test)]
pub(super) fn ground_contact(piece: &mut ObjectState) {
    weightless_ceiling_contact(piece);
    ground_contact_cached(piece, &mut ContactResponse::default());
}
pub(super) struct SampledContact {
    pub(super) normal: [f64; 3],
    pub(super) penetration: f64,
    pub(super) arms: Vec<[f64; 3]>,
}
pub(super) fn sampled_contact(piece: &ObjectState) -> Option<SampledContact> {
    let Surface::Sampled {
        body,
        query,
        point_query,
    } = piece.surface
    else {
        return None;
    };
    let hull = piece.hull.as_ref()?;
    let mut points = Vec::with_capacity(hull.vertices.len());
    let mut deepest = f64::INFINITY;
    let center_normal = query(body, piece.position).normal;
    let mut witness = piece.position;
    for vertex in &hull.vertices {
        let arm = convex::rotate(piece.orientation, scale(*vertex, piece.authored_mesh_scale));
        let point = add(piece.position, arm);
        let ground = point_query(body, point);
        let gap = dot(sub(point, ground), center_normal);
        if gap < deepest {
            deepest = gap;
            witness = point;
        }
        points.push((arm, gap));
    }
    let arms = points
        .into_iter()
        .filter(|(_, gap)| *gap <= deepest + 0.001 * piece.authored_mesh_scale)
        .map(|(arm, _)| arm)
        .collect();
    let normal = query(body, witness).normal;
    Some(SampledContact {
        normal,
        penetration: -deepest,
        arms,
    })
}
pub(super) fn ground_contact_cached(piece: &mut ObjectState, response: &mut ContactResponse) {
    weightless_ceiling_contact(piece);
    let Some(ground) = piece.surface.ground(piece.position) else {
        *response = ContactResponse::default();
        return;
    };
    let sampled = sampled_contact(piece);
    let up = sampled.as_ref().map_or(ground.normal, |c| c.normal);
    piece.ground_support_meters = piece.hull.as_ref().map_or(piece.radius, |hull| {
        hull.support(
            piece.orientation,
            piece.authored_mesh_scale,
            scale(up, -1.0),
        )
    });
    let penetration = sampled.as_ref().map_or_else(
        || {
            let gap = if let Surface::Sphere(body) = piece.surface {
                length(sub(piece.position, body.center)) - body.radius
            } else {
                dot(sub(piece.position, ground.point), up)
            };
            if let Surface::Sphere(body) = piece.surface {
                body.radius + piece.ground_support_meters - length(sub(piece.position, body.center))
            } else {
                piece.ground_support_meters - gap
            }
        },
        |c| c.penetration,
    );
    if penetration < -CONTACT_BIAS_METERS {
        *response = ContactResponse::default();
        return;
    }
    piece.position = add(piece.position, scale(up, penetration.max(0.0)));
    {
        let arms = sampled.map_or_else(
            || {
                piece.hull.as_ref().map_or_else(
                    || vec![scale(up, -piece.radius)],
                    |hull| {
                        hull.ground_arms(
                            piece.orientation,
                            piece.authored_mesh_scale,
                            scale(up, -1.0),
                        )
                    },
                )
            },
            |c| c.arms,
        );
        let paired_arms: Vec<_> = arms.iter().map(|&arm| (arm, [0.0; 3])).collect();
        response.prepare(up, &paired_arms, |_| 0.0);
        if std::mem::take(&mut response.pending_warm_start) {
            piece.angular_velocity = add(
                piece.angular_velocity,
                scale(response.rolling, inverse_inertia(piece)),
            );
            for (i, &arm) in arms.iter().enumerate() {
                apply_impulse(
                    piece,
                    arm,
                    add(scale(up, response.impulses[i]), response.friction[i]),
                );
            }
        }
        let normal_impulses = &mut response.impulses;
        // Accumulate unilateral impulses within this face solve. Later corners
        // can undo excess impulses while every corner's total stays nonnegative.
        // Positive-only corner impulses otherwise eject and rock resting faces.
        let friction_impulses = &mut response.friction;
        for _ in 0..16 {
            let before = iteration_state(
                [piece.velocity, piece.angular_velocity],
                normal_impulses,
                friction_impulses,
                response.rolling,
            );
            for (index, &arm) in arms.iter().enumerate() {
                let approach = dot(add(piece.velocity, cross(piece.angular_velocity, arm)), up);
                let denominator = 1.0 / piece.mass_kg
                    + dot(cross(arm, up), cross(arm, up)) * inverse_inertia(piece);
                let previous = normal_impulses[index];
                normal_impulses[index] = (previous - approach / denominator).max(0.0);
                apply_impulse(piece, arm, scale(up, normal_impulses[index] - previous));
            }
            for (i, &arm) in arms.iter().enumerate() {
                let velocity = add(piece.velocity, cross(piece.angular_velocity, arm));
                let denominator = |direction| {
                    1.0 / piece.mass_kg
                        + dot(cross(arm, direction), cross(arm, direction)) * inverse_inertia(piece)
                };
                let impulse = friction_delta(
                    velocity,
                    up,
                    &mut friction_impulses[i],
                    normal_impulses[i] * 0.65,
                    denominator,
                );
                apply_impulse(piece, arm, impulse);
            }
            let torque = rolling_delta(
                piece.angular_velocity,
                &mut response.rolling,
                ROLLING_RESISTANCE * normal_impulses.iter().sum::<f64>() * piece.radius,
                inverse_inertia(piece),
            );
            piece.angular_velocity = add(
                piece.angular_velocity,
                scale(torque, inverse_inertia(piece)),
            );
            if before.is_some()
                && before
                    == iteration_state(
                        [piece.velocity, piece.angular_velocity],
                        normal_impulses,
                        friction_impulses,
                        response.rolling,
                    )
            {
                break;
            }
        }
    }
}

// A convex body already supported by a rigid deck cannot resolve another
// contact by moving down through that deck. Project the positional response
// onto its available directions and give the remaining separation to its
// neighbor. Previously ground projection undid that share of every correction,
// leaving loaded stacks to compact and chatter over repeated passes. Physical
// mass/inertia and velocity impulses are unchanged; this is a support constraint.
pub(super) fn correction_direction(piece: &ObjectState, direction: [f64; 3]) -> [f64; 3] {
    if let Surface::Floor { height_meters } | Surface::UnpoweredFloor { height_meters, .. } =
        piece.surface
        && piece.position[1] - height_meters - piece.ground_support_meters <= CONTACT_BIAS_METERS
        && direction[1] < 0.0
    {
        [direction[0], 0.0, direction[2]]
    } else {
        direction
    }
}

pub(super) fn inverse_inertia(piece: &ObjectState) -> f64 {
    1.0 / (0.4 * piece.mass_kg * piece.radius * piece.radius)
}
pub(super) fn apply_impulse(piece: &mut ObjectState, arm: [f64; 3], impulse: [f64; 3]) {
    piece.velocity = add(piece.velocity, scale(impulse, 1.0 / piece.mass_kg));
    piece.angular_velocity = add(
        piece.angular_velocity,
        scale(cross(arm, impulse), inverse_inertia(piece)),
    );
}
// Project each accumulated tangential impulse onto its Coulomb disk. Distributed
// contacts resist twist as well as translation; later iterations can undo an
// earlier corner's excess impulse without exceeding its normal-load budget.
pub(super) fn friction_delta(
    relative: [f64; 3],
    normal: [f64; 3],
    previous: &mut [f64; 3],
    limit: f64,
    denominator: impl Fn([f64; 3]) -> f64,
) -> [f64; 3] {
    let tangent = sub(relative, scale(normal, dot(relative, normal)));
    let speed = length(tangent);
    let mut candidate = *previous;
    if speed > 1e-9 {
        candidate = sub(
            candidate,
            scale(tangent, 1.0 / denominator(scale(tangent, 1.0 / speed))),
        );
    }
    let magnitude = length(candidate);
    if magnitude > limit {
        candidate = scale(candidate, limit / magnitude);
    }
    let delta = sub(candidate, *previous);
    *previous = candidate;
    delta
}

#[cfg(test)]
pub(super) fn pair_impulse(a: &mut ObjectState, b: &mut ObjectState, normal: [f64; 3]) {
    pair_impulse_cached(a, b, normal, &mut ContactResponse::default());
}

pub(super) fn pair_impulse_cached(
    a: &mut ObjectState,
    b: &mut ObjectState,
    normal: [f64; 3],
    response: &mut ContactResponse,
) {
    let inverse_mass_a = 1.0 / a.mass_kg;
    let inverse_mass_b = 1.0 / b.mass_kg;
    let inertia_a = inverse_inertia(a);
    let inertia_b = inverse_inertia(b);
    {
        let arms = if a.hull.is_some() && b.hull.is_some() {
            convex::contact_manifold(a, b, normal)
        } else {
            vec![(scale(normal, a.radius), scale(normal, -b.radius))]
        };
        if !arms.is_empty() {
            let relative = |a: &ObjectState, b: &ObjectState, aa, ba| {
                sub(
                    add(b.velocity, cross(b.angular_velocity, ba)),
                    add(a.velocity, cross(a.angular_velocity, aa)),
                )
            };
            response.prepare(normal, &arms, |i| {
                let approach = dot(relative(a, b, arms[i].0, arms[i].1), normal);
                if approach < -0.5 {
                    -RESTITUTION * approach
                } else {
                    0.0
                }
            });
            if std::mem::take(&mut response.pending_warm_start) {
                a.angular_velocity = sub(a.angular_velocity, scale(response.rolling, inertia_a));
                b.angular_velocity = add(b.angular_velocity, scale(response.rolling, inertia_b));
                for (i, &(aa, ba)) in arms.iter().enumerate() {
                    let impulse = add(scale(normal, response.impulses[i]), response.friction[i]);
                    apply_impulse(a, aa, scale(impulse, -1.0));
                    apply_impulse(b, ba, impulse);
                }
            }
            let targets = &response.targets;
            let impulses = &mut response.impulses;
            let friction_impulses = &mut response.friction;
            for _ in 0..16 {
                let before = iteration_state(
                    [
                        a.velocity,
                        a.angular_velocity,
                        b.velocity,
                        b.angular_velocity,
                    ],
                    impulses,
                    friction_impulses,
                    response.rolling,
                );
                for (i, &(aa, ba)) in arms.iter().enumerate() {
                    let denominator = inverse_mass_a
                        + inverse_mass_b
                        + dot(cross(aa, normal), cross(aa, normal)) * inertia_a
                        + dot(cross(ba, normal), cross(ba, normal)) * inertia_b;
                    let previous = impulses[i];
                    impulses[i] = (previous
                        + (targets[i] - dot(relative(a, b, aa, ba), normal)) / denominator)
                        .max(0.0);
                    let impulse = scale(normal, impulses[i] - previous);
                    apply_impulse(a, aa, scale(impulse, -1.0));
                    apply_impulse(b, ba, impulse);
                }
                for (i, &(aa, ba)) in arms.iter().enumerate() {
                    let velocity = relative(a, b, aa, ba);
                    let denominator = |direction| {
                        inverse_mass_a
                            + inverse_mass_b
                            + dot(cross(aa, direction), cross(aa, direction)) * inertia_a
                            + dot(cross(ba, direction), cross(ba, direction)) * inertia_b
                    };
                    let impulse = friction_delta(
                        velocity,
                        normal,
                        &mut friction_impulses[i],
                        impulses[i] * 0.65,
                        denominator,
                    );
                    apply_impulse(a, aa, scale(impulse, -1.0));
                    apply_impulse(b, ba, impulse);
                }
                let torque = rolling_delta(
                    sub(b.angular_velocity, a.angular_velocity),
                    &mut response.rolling,
                    ROLLING_RESISTANCE * impulses.iter().sum::<f64>() * a.radius.min(b.radius),
                    inertia_a + inertia_b,
                );
                a.angular_velocity = sub(a.angular_velocity, scale(torque, inertia_a));
                b.angular_velocity = add(b.angular_velocity, scale(torque, inertia_b));
                if before.is_some()
                    && before
                        == iteration_state(
                            [
                                a.velocity,
                                a.angular_velocity,
                                b.velocity,
                                b.angular_velocity,
                            ],
                            impulses,
                            friction_impulses,
                            response.rolling,
                        )
                {
                    break;
                }
            }
            return;
        }
    }
    let (aa, ba) = if a.hull.is_some() && b.hull.is_some() {
        convex::contact_arms(a, b, normal)
    } else {
        (scale(normal, a.radius), scale(normal, -b.radius))
    };
    let relative = sub(
        add(b.velocity, cross(b.angular_velocity, ba)),
        add(a.velocity, cross(a.angular_velocity, aa)),
    );
    let approach = dot(relative, normal);
    if approach >= 0.0 {
        return;
    }
    let denom = |direction| {
        inverse_mass_a
            + inverse_mass_b
            + dot(cross(aa, direction), cross(aa, direction)) * inertia_a
            + dot(cross(ba, direction), cross(ba, direction)) * inertia_b
    };
    let restitution = if a.hull.is_none() || approach < -0.5 {
        RESTITUTION
    } else {
        0.0
    };
    let magnitude = -(1.0 + restitution) * approach / denom(normal);
    let tangent = sub(relative, scale(normal, approach));
    let speed = length(tangent);
    let friction = if speed > 1e-9 {
        let direction = scale(tangent, 1.0 / speed);
        scale(direction, -(speed / denom(direction)).min(magnitude * 0.65))
    } else {
        [0.0; 3]
    };
    let impulse = add(scale(normal, magnitude), friction);
    apply_impulse(a, aa, scale(impulse, -1.0));
    apply_impulse(b, ba, impulse);
}

impl ContactResponse {
    pub(super) fn validate_save(&self) -> Result<(), salimon_persistence::SaveError> {
        let n = self.arms.len();
        if n != self.impulses.len()
            || n != self.friction.len()
            || n != self.targets.len()
            || self
                .normal
                .iter()
                .chain(self.rolling.iter())
                .chain(self.arms.iter().flat_map(|(a, b)| a.iter().chain(b.iter())))
                .chain(self.impulses.iter())
                .chain(self.friction.iter().flatten())
                .chain(self.targets.iter())
                .any(|v| !v.is_finite())
        {
            return Err(salimon_persistence::invalid("contact impulse"));
        }
        Ok(())
    }
}
