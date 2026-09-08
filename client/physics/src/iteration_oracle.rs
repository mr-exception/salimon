//! Frozen pre-#592 sixteen-iteration response oracle. Do not route production here.
use super::*;
fn ground_reference(piece: &mut ObjectState, response: &mut ContactResponse) {
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
        }
    }
}
fn pair_reference(
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
fn body(seed: usize) -> ObjectState {
    let radius = [0.00023, 0.1, 0.4, 1.][seed % 4];
    ObjectState {
        position: [0., radius, 0.],
        velocity: [seed as f64 * 0.013 - 0.2, -0.1, 0.07],
        radius,
        ground_support_meters: radius,
        surface: Surface::Floor { height_meters: 0. },
        hull: None,
        authored_mesh_scale: 1.,
        orientation: [0., 0., 0., 1.],
        angular_velocity: [0.02, -0.03, seed as f64 * 0.001],
        mass_kg: [0.005, 10., 1200., 1e6][seed % 4],
    }
}
#[test]
fn exact_fixed_point_elision_matches_retained_sixteen_iteration_response_and_warm_continuation() {
    for seed in 0..64 {
        let mut a = body(seed);
        let mut b = body(seed + 3);
        b.position[0] = a.radius + b.radius;
        if seed % 3 == 0 {
            a.surface = Surface::Sphere(SphereSurface {
                center: [0., -10., 0.],
                radius: 10.,
            });
            a.position[0] = 0.1;
        }
        if seed % 3 == 1 {
            a.surface = Surface::UnpoweredFloor {
                height_meters: 0.,
                ceiling_meters: 4.,
            };
        }
        if seed % 5 == 0 {
            let hull = Arc::new(ConvexHull::new([-0.5, 0.5].into_iter().flat_map(|x| {
                [-0.5, 0.5]
                    .into_iter()
                    .flat_map(move |y| [-0.5, 0.5].into_iter().map(move |z| [x, y, z]))
            })));
            a.hull = Some(hull.clone());
            b.hull = Some(hull);
            a.authored_mesh_scale = a.radius;
            b.authored_mesh_scale = b.radius;
        }
        let (mut old_a, mut old_b) = (a.clone(), b.clone());
        let (mut pair, mut old_pair) = (ContactResponse::default(), ContactResponse::default());
        let (mut ground, mut old_ground) = (ContactResponse::default(), ContactResponse::default());
        for tick in 0..32 {
            if tick % 4 == 0 {
                pair.rearm(1.);
                old_pair.rearm(1.);
                ground.rearm(1.);
                old_ground.rearm(1.);
            }
            let raw = [[1., 0., 0.], [0.2, 0.7, -0.3], [-0.1, 0.3, 0.9]][tick % 3];
            let normal = scale(raw, 1. / length(raw));
            pair_impulse_cached(&mut a, &mut b, normal, &mut pair);
            pair_reference(&mut old_a, &mut old_b, normal, &mut old_pair);
            ground_contact_cached(&mut a, &mut ground);
            ground_reference(&mut old_a, &mut old_ground);
            assert_eq!(
                serde_json::to_vec(&(&a, &b, &pair, &ground)).unwrap(),
                serde_json::to_vec(&(&old_a, &old_b, &old_pair, &old_ground)).unwrap(),
                "seed {seed}, tick {tick}"
            );
        }
    }
}
