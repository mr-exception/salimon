//! Authored physical evidence, recovered claims and voluntary chart admission.
//! These fixed sites are fiction on Vesper, never measured Gaia objects.
use super::{GameplaySession, ViewMode, recovery::Recovery};
use salimon_math::{add, dot, length, scale, sub};
use salimon_persistence::{SaveError, Validate, invalid};
use salimon_universe::{
    CatalogId,
    scanner::{
        Position,
        chart::{Chart, CoordinateUncertainty, Lead},
    },
};

pub(super) const RECORDS: [&str; 2] = ["received.damaged-01", "received.damaged-02"];
const SOURCES: [&str; 2] = [
    "scout.received.damaged-01.location-v1",
    "scout.received.damaged-02.location-v1",
];
pub(super) const PAYLOADS: [&str; 2] = [
    "RECEIVED CLAIM / sender and transmission time unknown.\nDistress: accident at the attached Vesper surface fix.\nPlease investigate the damaged relay. +/-25m.\nPacket frame: Vesper body-local, location-v1.\nA received claim, not proof of the traveler's visit.\nP in cockpit: chart; Enter: voluntary uncertain route.",
    "RECEIVED WARNING / sender and time unknown.\nIt is a trap. Do not take the passage.\nSeparate origin: Vesper remote relay, +/-25m.\nPacket frame: Vesper body-local, location-v1.\n[irrecoverable gap: sender, motive, chronology]\nNo proof of passage or of the warning's truth.\nP in cockpit: chart; Enter: voluntary uncertain route.",
];
const CLUES: [&str; 2] = [
    "OBSERVED: fractured relay casing; scored ceramic plate.\nLEGIBLE INSCRIPTION / UNVERIFIED CLAIM:\n'Passage may emerge near Absenat.'\nNo endpoint, working passage or transit is established.\nWho wrote this, and whether it is true, remain unknown.",
    "OBSERVED: detached transmitter housing, dark indicator.\nLEGIBLE LOCAL BUFFER / UNVERIFIED CLAIM:\n'Trap. Do not take the passage.' [remaining bytes lost]\nNo sender identity or motive survives.\nThis does not establish who sent the received warning.",
];
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct State {
    version: u32,
    inspected_utc_millis: [Option<u64>; 2],
    #[serde(skip)]
    pub(crate) reader: Option<usize>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            version: 1,
            inspected_utc_millis: [None; 2],
            reader: None,
        }
    }
}
impl Validate for State {
    fn validate(&self) -> Result<(), SaveError> {
        if self.version != 1 {
            return Err(invalid("narrative site publication version"));
        }
        Ok(())
    }
}
impl State {
    pub(super) fn validate_clock(&self, now: u64) -> Result<(), SaveError> {
        if self.inspected_utc_millis.iter().flatten().any(|t| *t > now) {
            return Err(invalid("physical narrative inspection ahead of clock"));
        }
        Ok(())
    }
}
pub(super) fn restore(source: Option<State>, schema: u32) -> Result<State, SaveError> {
    match (schema, source) {
        (1.., Some(s)) => Ok(s),
        (1..=30, None) => Ok(State::default()),
        _ => Err(invalid("narrative source/schema mismatch")),
    }
}
pub(super) fn site_name(id: &CatalogId) -> Option<&'static str> {
    (0..2)
        .find(|i| *id == site_id(*i))
        .map(|i| ["DISTRESS SIGNAL", "WARNING ORIGIN"][i])
}
fn site_id(i: usize) -> CatalogId {
    CatalogId::from_source(
        "salimon-site",
        ["vesper-distress-v1", "vesper-warning-v1"][i],
    )
    .expect("fixed site identity")
}
/// Source and physical rendering use exactly the same terrain query. No LOD or
/// per-run seed affects these anchors. The second origin is ~14 km away.
pub(crate) fn site_position(i: usize) -> [f64; 3] {
    let xz = [[65., 0.], [12000., -8000.]][i];
    let body = salimon_world::canonical::BODIES[0];
    let n = [xz[0], body.radius_meters, xz[1]];
    let direction = scale(n, 1. / length(n));
    let height = salimon_world::terrain::TerrainMode::Canonical.height(body, direction);
    scale(direction, body.radius_meters + height + 0.4)
}
fn lead(i: usize, now: u64) -> Lead {
    Lead {
        position: Position::BodyLocal {
            body: salimon_world::canonical::START.manifest().planet.id.clone(),
            meters: site_position(i),
        },
        source: SOURCES[i].into(),
        received_utc_millis: now,
        inferred: false,
        uncertainty: CoordinateUncertainty::RadiusMeters { meters: 25. },
    }
}
pub(super) fn validate_leads(chart: &Chart, recovery: &Recovery) -> Result<(), SaveError> {
    for entry in chart.entries() {
        if entry.id.as_str().starts_with("salimon-site:") {
            let i = (0..2)
                .find(|i| entry.id == site_id(*i))
                .ok_or_else(|| invalid("unknown narrative site"))?;
            if !recovery.complete(i)
                || entry.leads().len() != 1
                || entry.leads()[0] != lead(i, entry.leads()[0].received_utc_millis)
            {
                return Err(invalid("narrative lead differs from recovered packet"));
            }
        }
    }
    Ok(())
}
impl GameplaySession {
    pub(super) fn publish_narrative_leads(&mut self) {
        if self.mining.terrain != salimon_world::terrain::TerrainMode::Canonical {
            return;
        }
        for i in 0..2 {
            if self.recovery.complete(i)
                && self.chart.get(&site_id(i)).is_none()
                && let Err(e) =
                    self.chart_receive_lead(site_id(i), lead(i, self.scanner.utc_millis))
            {
                log::warn!("recovered location publication: {e}");
                self.feedback =
                    Some("Recovered location could not be charted; retained in Archive");
            }
        }
    }
    fn narrative_target(&self) -> Option<usize> {
        if self.view_mode != ViewMode::Gameplay
            || self.mining.terrain != salimon_world::terrain::TerrainMode::Canonical
            || self.is_paused()
            || self.survival.snapshot().terminal
            || self.character.location() != salimon_character::CharacterLocation::Surface
        {
            return None;
        }
        let c = self.snapshot().character;
        let look = sub(c.look_target_meters, c.eye_position_meters);
        (0..2).find(|i| {
            let v = sub(site_position(*i), c.eye_position_meters);
            let distance = length(v);
            distance > 0.
                && distance <= 3.
                && dot(v, look) / (distance * length(look)) >= 0.94
                && salimon_world::mining::MiningRay::new(
                    salimon_world::WorldPosition::new(
                        c.eye_position_meters[0],
                        c.eye_position_meters[1],
                        c.eye_position_meters[2],
                    ),
                    salimon_world::WorldPosition::new(
                        site_position(*i)[0],
                        site_position(*i)[1],
                        site_position(*i)[2],
                    ),
                )
                .is_some_and(|ray| {
                    salimon_world::mining::terrain_obstruction(ray, self.mining.terrain, distance)
                        .is_none_or(|d| d >= distance - 0.5)
                })
        })
    }
    pub(crate) fn inspect_narrative_site(&mut self) -> bool {
        if self.narrative.reader.take().is_some() {
            return true;
        }
        let Some(i) = self.narrative_target() else {
            return false;
        };
        if !self.admit_survival_action(salimon_survival::restrictions::Action::Menu) {
            return true;
        }
        self.clear_held_controls();
        self.narrative.inspected_utc_millis[i].get_or_insert(self.scanner.utc_millis);
        self.narrative.reader = Some(i);
        true
    }
    pub(crate) fn narrative_text(&self) -> Option<String> {
        let i = self.narrative.reader?;
        if self.is_paused() || self.narrative_target() != Some(i) {
            return None;
        }
        Some(format!(
            "PHYSICAL EVIDENCE / {}\n{}\nSaved to Archive. E: close",
            ["DISTRESS SITE", "REMOTE WARNING ORIGIN"][i],
            CLUES[i]
        ))
    }
    pub(super) fn narrative_archive_clue(&self, record: &str) -> String {
        RECORDS
            .iter()
            .position(|r| *r == record)
            .filter(|i| self.narrative.inspected_utc_millis[*i].is_some())
            .map_or_else(String::new, |i| {
                format!("\nFIELD INSPECTION RETAINED\n{}", CLUES[i])
            })
    }
    pub(crate) fn narrative_hint(&self) -> Option<String> {
        self.narrative_target().map(|i| {
            format!(
                "E: inspect {}",
                ["fractured relay", "remote transmitter"][i]
            )
        })
    }
    pub(crate) fn narrative_geometry(&self) -> Vec<salimon_renderer::SceneInstance> {
        if self.mining.terrain != salimon_world::terrain::TerrainMode::Canonical {
            return Vec::new();
        }
        // Small ruined equipment with no traversable interior or blocking volume.
        // Kept below knee height; inspection and location share the same anchor.
        (0..2)
            .flat_map(|i| {
                let p = site_position(i);
                [
                    salimon_renderer::SceneInstance {
                        center_meters: p,
                        orientation: [0., 0., 0., 1.],
                        half_extents_meters: [0.75, 0.25, 0.5],
                        color: [0.23, 0.25, 0.28, 1.],
                    },
                    salimon_renderer::SceneInstance {
                        center_meters: add(p, [0.25, 0.28, 0.]),
                        orientation: [0., 0., 0.17, 0.985444],
                        half_extents_meters: [0.26, 0.06, 0.35],
                        color: [0.48, 0.37, 0.24, 1.],
                    },
                ]
            })
            .collect()
    }
}
#[cfg(test)]
mod tests;

/// Opt-in evidence initial conditions. Declared placement/resources never claim
/// a walked route, earned inventory or an interstellar trip.
pub(crate) fn initialize_fixture(
    s: &mut GameplaySession,
    site: Option<usize>,
) -> Result<(), String> {
    *s = GameplaySession::new_canonical_run()?;
    if let Some(i) = site {
        let p = site_position(i);
        let surface = s.terrain_frame(s.ship.snapshot());
        s.character =
            salimon_character::CharacterController::on_surface(surface, add(p, [1.8, 0., 0.]))
                .map_err(str::to_owned)?;
        let frame = super::frames::character_ship_frame(s.ship.snapshot().pose);
        let c = s.character.snapshot(frame, surface);
        let v = sub(p, c.eye_position_meters);
        let yaw = dot(v, frame.axes[2]).atan2(dot(v, frame.axes[0]));
        let pitch = (dot(v, c.up.map(f64::from)) / length(v)).asin();
        s.character.apply_mouse_delta(yaw / 0.0022, -pitch / 0.0022);
    } else {
        use salimon_fabrication::{FabricationCommand as C, RecipeId};
        for resource in [
            salimon_world::resources::ResourceId::IronOre,
            salimon_world::resources::ResourceId::SilicateRock,
        ] {
            s.execute_fabricator(C::Insert(
                salimon_world::resources::RawMaterial::new(resource, 2.)
                    .map_err(|e| format!("{e:?}"))?,
            ))
            .map_err(|e| e.to_string())?;
        }
        s.execute_fabricator(C::Enqueue {
            recipe: RecipeId::new("data-recovery-module").map_err(|e| e.to_string())?,
            quantity: 1,
        })
        .map_err(|e| e.to_string())?;
        s.advance_fabrication(std::time::Duration::from_secs(60));
        s.collect_fabricator_output();
        for record in RECORDS {
            s.toggle_recovery(record);
            for _ in 0..100 {
                s.advance_recovery(std::time::Duration::from_secs(10));
            }
        }
        let mut source =
            serde_json::to_value(s.character.save_state()).map_err(|e| e.to_string())?;
        source["position"] = serde_json::json!({"Inside":{"local":[-3.,1.7,-2.3]}});
        source["yaw_radians"] = serde_json::json!(-std::f64::consts::FRAC_PI_2);
        source["pitch_radians"] = serde_json::json!(0.);
        s.character = salimon_character::CharacterController::from_save(
            serde_json::from_value(source).map_err(|e| e.to_string())?,
            None,
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}
