use super::*;

pub(crate) fn initialize_fixture(s: &mut GameplaySession, embedded: bool) -> Result<(), String> {
    *s = GameplaySession::default();
    s.biology.fixture = true;
    let body = *salimon_world::CELESTIAL_BODIES
        .iter()
        .find(|b| b.id == salimon_world::CelestialBodyId::Moon)
        .ok_or("fixture body")?;
    s.enable_terrain(body, [0., 1., 0.])?;
    position_fixture_observer(s, embedded)
}
pub(crate) fn position_fixture_observer(
    s: &mut GameplaySession,
    embedded: bool,
) -> Result<(), String> {
    let source = FIXTURE
        .catalog
        .sources()
        .iter()
        .find(|s| {
            s.method
                == if embedded {
                    Method::Embedded
                } else {
                    Method::Detach
                }
        })
        .ok_or("fixture source")?;
    let object = FIXTURE
        .population
        .objects()
        .iter()
        .find(|o| o.id == source.key.organism)
        .ok_or("fixture organism")?;
    let base = origin(true);
    let center = std::array::from_fn(|i| source.center[i] + base[i]);
    let lateral = salimon_math::sub(source.center, object.position_meters);
    let up = object.axes[1];
    let along = salimon_math::dot(lateral, up);
    let tangent: [f64; 3] = std::array::from_fn(|i| lateral[i] - up[i] * along);
    let norm = salimon_math::length(tangent);
    let tangent = if norm > 1e-6 {
        tangent.map(|v| v / norm)
    } else {
        object.axes[0]
    };
    let frame = s.terrain_frame(s.ship.snapshot());
    let hint = std::array::from_fn(|i| center[i] + tangent[i] * 2.8);
    s.character =
        salimon_character::CharacterController::on_surface(frame, hint).map_err(str::to_owned)?;
    let ship_frame = crate::session::frames::character_ship_frame(s.ship.snapshot().pose);
    let eye = s.character.snapshot(ship_frame, frame);
    let d = salimon_math::sub(center, eye.eye_position_meters);
    let n = salimon_math::length(d);
    let d = d.map(|v| v / n);
    let up = eye.up.map(f64::from);
    let project = |v: [f64; 3]| {
        let dot = salimon_math::dot(v, up);
        std::array::from_fn(|i| v[i] - dot * up[i])
    };
    let a = project(ship_frame.axes[0]);
    let b = project(ship_frame.axes[2]);
    let aa = salimon_math::dot(a, a);
    let bb = salimon_math::dot(b, b);
    let ab = salimon_math::dot(a, b);
    let x = salimon_math::dot(d, a);
    let y = salimon_math::dot(d, b);
    let yaw = (y * aa - x * ab).atan2(x * bb - y * ab);
    let pitch = salimon_math::dot(d, eye.up.map(f64::from))
        .clamp(-1., 1.)
        .asin();
    s.character.apply_mouse_delta(yaw / 0.0022, -pitch / 0.0022);
    s.reset_presentation();
    Ok(())
}
