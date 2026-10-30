//! Physical Scout chart composition; personal notes never mutate world sources.
use super::GameplaySession;
use salimon_persistence::{SaveError, invalid};
use salimon_universe::{
    surface_chart::{Chart, InspectionSample},
    surface_observations::Book,
};
#[derive(Default)]
pub(crate) struct State {
    pub(crate) source: Chart,
    pub(crate) open: bool,
    pub(crate) editing: bool,
    draft: String,
    selected: usize,
}
pub(crate) fn validate(
    source: &Chart,
    book: &Book,
    elapsed: std::time::Duration,
) -> Result<(), SaveError> {
    source.validate(book).map_err(|e| invalid(e.to_string()))?;
    let now = super::field_marks::epoch(elapsed)?;
    for a in source.entries.values() {
        if a.edits
            .iter()
            .any(|e| now.seconds_since(e.epoch).map_or(true, |s| s < 0.))
        {
            return Err(invalid("surface edit ahead of shared clock"));
        }
        for p in &a.inspections {
            let published = match p.kind.as_str() {
                "geological" => (p.publication == salimon_world::canonical::LANDMARKS.identity())
                    .then(|| {
                        salimon_world::canonical::LANDMARKS
                            .sites()
                            .iter()
                            .find(|s| s.id == p.site_id)
                            .map(|s| (s.position_meters, s.radius_meters))
                    })
                    .flatten(),
                "biological" => (p.publication
                    == salimon_world::canonical::NATURAL_SITES.identity())
                .then(|| {
                    salimon_world::canonical::NATURAL_SITES
                        .sites()
                        .iter()
                        .find(|s| s.id == p.site_id)
                        .map(|s| {
                            (
                                s.position_meters,
                                (0..3)
                                    .map(|i| {
                                        (s.bounds[2 * i] - s.position_meters[i])
                                            .abs()
                                            .max((s.bounds[2 * i + 1] - s.position_meters[i]).abs())
                                            .powi(2)
                                    })
                                    .sum::<f64>()
                                    .sqrt(),
                            )
                        })
                })
                .flatten(),
                _ => None,
            };
            if !published.is_some_and(|(actual, extent)| {
                p.location.position_meters.is_some_and(|measured| {
                    salimon_math::length(salimon_math::sub(actual, measured))
                        <= extent + p.location.radius_meters.unwrap_or(0.)
                })
            }) {
                return Err(SaveError::IncompatibleCatalog(
                    "inspected site publication/identity".into(),
                ));
            }
            if now.seconds_since(p.epoch).map_or(true, |s| s < 0.) {
                return Err(invalid("surface inspection future epoch"));
            }
        }
    }
    if source
        .targets
        .iter()
        .any(|t| now.seconds_since(t.epoch).map_or(true, |s| s < 0.))
    {
        return Err(invalid("surface target future epoch"));
    }
    Ok(())
}
pub(crate) fn restore(
    source: Option<Chart>,
    schema: u32,
    book: Option<&Book>,
    elapsed: std::time::Duration,
) -> Result<State, SaveError> {
    match (source, book, schema) {
        (Some(source), Some(book), _) => {
            validate(&source, book, elapsed)?;
            Ok(State {
                source,
                ..Default::default()
            })
        }
        (None, Some(book), 1..=27) => {
            let mut source = Chart::default();
            source.sync(book).map_err(|e| invalid(e.to_string()))?;
            Ok(State {
                source,
                ..Default::default()
            })
        }
        (None, None, _) => Ok(State::default()),
        _ => Err(invalid(
            "surface chart missing/incompatible with schema/world",
        )),
    }
}
impl GameplaySession {
    pub(crate) fn surface_capture(&self) -> Result<Option<Chart>, SaveError> {
        let Some(book) = self.field_capture()? else {
            return Ok(None);
        };
        let mut chart = self.surface_chart.source.clone();
        chart.sync(&book).map_err(|e| invalid(e.to_string()))?;
        validate(&chart, &book, self.elapsed)?;
        Ok(Some(chart))
    }
    fn surface_available(&self) -> bool {
        self.scanner_controls_available()
            && !self.building.active
            && self.ship.snapshot().power_available
    }
    fn surface_selected(&self, chart: &Chart) -> Option<u64> {
        let ids = chart.active_ids();
        ids.get(self.surface_chart.selected.checked_rem(ids.len())?)
            .copied()
    }
    pub(crate) fn surface_key(&mut self, key: super::input::Key, pressed: bool) -> bool {
        use super::input::Key;
        if key == Key::SurfaceInspect {
            if pressed {
                self.inspect_surface_mark();
            }
            return true;
        }
        if key == Key::SurfaceChart {
            if pressed && (self.surface_chart.open || self.surface_available()) {
                self.surface_chart.open = !self.surface_chart.open;
                self.surface_chart.editing = false;
                self.scanner.chart_mode = false;
                self.clear_held_controls();
            }
            return true;
        }
        if !self.surface_chart.open || !pressed {
            return false;
        }
        if key == Key::Cancel {
            self.surface_chart.open = false;
            self.surface_chart.editing = false;
            return true;
        }
        if !self.surface_available() {
            return true;
        }
        let Ok(Some(book)) = self.field_capture() else {
            return true;
        };
        let Ok(Some(mut chart)) = self.surface_capture() else {
            return true;
        };
        let Some(id) = self.surface_selected(&chart) else {
            return true;
        };
        let Ok(now) = super::field_marks::epoch(self.elapsed) else {
            return true;
        };
        let was_editing = self.surface_chart.editing;
        let result = if self.surface_chart.editing {
            match key {
                Key::NoteBackspace => {
                    self.surface_chart.draft.pop();
                    Ok(())
                }
                Key::Confirm | Key::ConfirmAlternate => {
                    let result =
                        chart.edit(&book, id, self.surface_chart.draft.clone(), false, now);
                    if result.is_ok() {
                        self.surface_chart.editing = false;
                    }
                    result
                }
                _ => Ok(()),
            }
        } else {
            match key {
                Key::ScannerNext => {
                    self.surface_chart.selected = self.surface_chart.selected.wrapping_add(1);
                    Ok(())
                }
                Key::Recovery => {
                    self.surface_chart.draft = chart.entries[&id].note().to_owned();
                    self.surface_chart.editing = true;
                    Ok(())
                }
                Key::Delete => {
                    chart.edit(&book, id, chart.entries[&id].note().to_owned(), true, now)
                }
                Key::Confirm | Key::ConfirmAlternate => chart.route(&book, id, now),
                _ => Ok(()),
            }
        };
        if result.is_err() {
            self.feedback = Some("CHART ACTION DECLINED - CHECK RANGE, FIX OR HISTORY LIMIT");
        } else {
            self.surface_chart.source = chart;
            self.feedback = match key {
                Key::Delete if !was_editing => Some("PERSONAL MARK REMOVED"),
                Key::Confirm | Key::ConfirmAlternate if was_editing => Some("PERSONAL NOTE SAVED"),
                Key::Confirm | Key::ConfirmAlternate => Some("MANUAL TARGET SAVED - NO AUTOPILOT"),
                _ => self.feedback,
            };
        }
        true
    }
    pub(crate) fn surface_text(&mut self, text: &str) -> Result<(), SaveError> {
        if !self.surface_chart.open || !self.surface_chart.editing || !self.surface_available() {
            return Err(invalid("surface note editor unavailable"));
        }
        if text.chars().any(char::is_control)
            || self.surface_chart.draft.chars().count() + text.chars().count() > 256
        {
            return Err(invalid("surface note text/length"));
        }
        self.surface_chart.draft.push_str(text);
        Ok(())
    }
    fn inspect_surface_mark(&mut self) {
        if self.is_paused()
            || self.survival.snapshot().terminal
            || self.view_mode != super::ViewMode::Gameplay
            || self.wheel.open
            || self.archive_reader.open
            || self.crafting_ui.open
            || self.building.active
            || self.surface_chart.open
        {
            return;
        }
        if self.update_field_measurement().is_err() {
            return;
        }
        let Ok(Some(book)) = self.field_capture() else {
            return;
        };
        let Ok(Some(mut chart)) = self.surface_capture() else {
            return;
        };
        let Some(id) = chart.target.or_else(|| chart.active_ids().last().copied()) else {
            self.feedback = Some("CREATE A PERSONAL FIELD MARK FIRST");
            return;
        };
        let p = self.snapshot().character;
        let eye = p.eye_position_meters;
        let Some(ray) = salimon_world::mining::MiningRay::new(
            salimon_world::WorldPosition::new(eye[0], eye[1], eye[2]),
            salimon_world::WorldPosition::new(
                p.look_target_meters[0],
                p.look_target_meters[1],
                p.look_target_meters[2],
            ),
        ) else {
            return;
        };
        let point = |d: f64| std::array::from_fn(|i| eye[i] + ray.direction[i] * d);
        let terrain_hit = salimon_world::mining::terrain_obstruction(ray, self.mining.terrain, 10.);
        let mut candidate: Option<(String, [f64; 3], &str, String, f64)> = None;
        if let Some(d) = terrain_hit.filter(|d| *d <= 10.) {
            let hit = point(d);
            if let Ok(sites) = salimon_world::canonical::LANDMARKS.nearby(hit, 0.)
                && let Some(site) = sites.first()
            {
                candidate = Some((
                    site.id.clone(),
                    hit,
                    "geological",
                    salimon_world::canonical::LANDMARKS.identity().into(),
                    d,
                ));
            }
        }
        for site in salimon_world::canonical::NATURAL_SITES.sites() {
            for organism in salimon_world::canonical::ORGANISMS
                .objects()
                .iter()
                .filter(|o| site.organism_ids.contains(&o.id))
            {
                for part in &organism.parts {
                    if let Some(d) = salimon_character::cargo::box_sight_obstruction(
                        eye,
                        point(10.),
                        part.bounds,
                    ) && d <= 10.
                        && candidate.as_ref().is_none_or(|c| d < c.4)
                    {
                        candidate = Some((
                            site.id.clone(),
                            point(d),
                            "biological",
                            salimon_world::canonical::NATURAL_SITES.identity().into(),
                            d,
                        ));
                    }
                }
            }
        }
        let Some((site_id, site, kind, publication, distance)) = candidate else {
            self.feedback = Some("NO NEARBY AIMED CANONICAL SITE TO INSPECT");
            return;
        };
        let ship = self.ship.snapshot();
        let frame = super::frames::character_ship_frame(ship.pose);
        if terrain_hit.is_some_and(|d| d < distance - 0.25)
            || super::organisms::FIELD
                .sweep(eye, site, [0.; 3])
                .is_ok_and(|hit| hit.is_some_and(|h| h.fraction * distance < distance - 0.25))
            || salimon_character::ship_sight_obstruction(
                frame.world_to_local(eye),
                frame.world_to_local(site),
                ship.door_open_fraction >= 0.95,
            )
            .is_some_and(|d| d < distance - 0.25)
            || self.construction_solids().iter().any(|b| {
                salimon_character::cargo::box_sight_obstruction(
                    frame.world_to_local(eye),
                    frame.world_to_local(site),
                    *b,
                )
                .is_some_and(|d| d < distance - 0.25)
            })
        {
            self.feedback = Some("SITE OBSTRUCTED");
            return;
        }
        let Ok(epoch) = super::field_marks::epoch(self.elapsed) else {
            return;
        };
        if chart
            .inspect(
                &book,
                id,
                InspectionSample {
                    site_id,
                    publication,
                    kind: kind.into(),
                    site,
                    observer: eye,
                    epoch,
                },
            )
            .is_ok()
        {
            self.surface_chart.source = chart;
            self.feedback = Some("PHYSICAL SITE INSPECTED - SCIENTIFIC IDENTIFICATION UNCONFIRMED");
        } else {
            self.feedback = Some("SITE DOES NOT MATCH MARK OR NAVIGATION UNCERTAINTY");
        }
    }
    pub(crate) fn surface_hint(&self) -> Option<String> {
        if !self.surface_chart.open || !self.scanner_controls_available() {
            return None;
        }
        if !self.ship.snapshot().power_available {
            return Some("SURFACE CHART POWER OFF | BACKSLASH CLOSE".into());
        }
        let chart = self.surface_capture().ok()??;
        let book = self.field_capture().ok()??;
        let Some(id) = self.surface_selected(&chart) else {
            return Some("SURFACE CHART EMPTY | F10/F11 FIELD MARKS | BACKSLASH CLOSE".into());
        };
        let mark = book.marks().iter().find(|m| m.id == id)?;
        let a = &chart.entries[&id];
        if self.surface_chart.editing {
            return Some(format!(
                "PERSONAL NOTE: {}\nENTER SAVE | BACKSPACE EDIT | ESC CLOSE",
                self.surface_chart
                    .draft
                    .chars()
                    .rev()
                    .take(70)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect::<String>()
            ));
        }
        let location = a.inspected().map(|p| &p.location).unwrap_or(&mark.estimate);
        let quality = if mark.bearing.is_some() && a.inspected().is_none() {
            if mark
                .bearing_error_radians
                .is_some_and(|e| e >= std::f64::consts::FRAC_PI_2)
            {
                "BEARING UNCONSTRAINED - RANGE UNKNOWN"
            } else {
                "BEARING ONLY - RANGE UNKNOWN"
            }
        } else {
            location.quality()
        };
        let note = a.note().chars().take(28).collect::<String>();
        let navigation = if mark.bearing.is_some() && a.inspected().is_none() {
            "RANGE UNKNOWN".into()
        } else {
            location.position_meters.map_or("NO FIX".into(), |p| {
                let ship = self.ship.snapshot();
                let reference = ship
                    .pose
                    .position_meters
                    .map(|v| (v / book.config.grid_meters).round() * book.config.grid_meters);
                let v = salimon_math::sub(p, reference);
                let local = ship.pose.axes().map(|axis| salimon_math::dot(v, axis));
                format!(
                    "EST {:.0}m +/-{:.1}m AZ{:.0} EL{:.0}",
                    salimon_math::length(v),
                    location.radius_meters.unwrap_or(0.) + book.config.fix_error_meters,
                    local[2].atan2(local[0]).to_degrees(),
                    local[1].atan2(local[0].hypot(local[2])).to_degrees()
                )
            })
        };
        let target = chart
            .target
            .map_or("NO TARGET".into(), |id| format!("TARGET MARK {id}"));
        let uncertainty = location
            .radius_meters
            .map_or("NO FIX".into(), |r| format!("+/-{r:.1}m"));
        Some(format!(
            "SURFACE MARK {id}: {note} | {quality} {uncertainty}\n{} | HISTORY {} | M NEXT | R NOTE | DEL REMOVE\n{navigation} | {target} | ENTER TARGET - NO AUTOPILOT
COMMA INSPECT NEAR SITE | BACKSLASH CLOSE",
            if a.inspected().is_some() {
                "PHYSICAL SITE INSPECTED"
            } else {
                "PERSONAL OBSERVATION"
            },
            a.edits.len()
        ))
    }
    pub(crate) fn surface_geometry(
        &self,
        pose: salimon_ship::ShipPose,
    ) -> Vec<salimon_renderer::SceneInstance> {
        let mut contacts = vec![];
        let mut range = 10_000_f64;
        if let (Ok(Some(book)), Ok(Some(chart))) = (self.field_capture(), self.surface_capture())
            && let Some(id) = self.surface_selected(&chart)
            && let Some(mark) = book.marks().iter().find(|m| m.id == id)
        {
            let a = &chart.entries[&id];
            let location = a.inspected().map(|p| &p.location).unwrap_or(&mark.estimate);
            if let Some(origin) = location.position_meters {
                let positions = if mark.bearing.is_some()
                    && a.inspected().is_none()
                    && mark
                        .bearing_error_radians
                        .is_some_and(|e| e >= std::f64::consts::FRAC_PI_2)
                {
                    vec![]
                } else if let Some(direction) = mark.bearing.filter(|_| a.inspected().is_none()) {
                    let axis = if direction[1].abs() < 0.9 {
                        [0., 1., 0.]
                    } else {
                        [1., 0., 0.]
                    };
                    let x = salimon_math::cross(direction, axis);
                    let length = salimon_math::length(x);
                    let x = x.map(|v| v / length);
                    let y = salimon_math::cross(direction, x);
                    let mut out = vec![];
                    for d in [100., 1000., 10_000.] {
                        for i in 0..8 {
                            let a = f64::from(i) * std::f64::consts::TAU / 8.;
                            let width = d * mark.bearing_error_radians.unwrap_or(0.).tan()
                                + location.radius_meters.unwrap_or(0.)
                                + book.config.fix_error_meters;
                            out.push(std::array::from_fn(|j| {
                                origin[j]
                                    + direction[j] * d
                                    + width * (x[j] * a.cos() + y[j] * a.sin())
                            }));
                        }
                    }
                    out
                } else {
                    let mut out = vec![origin];
                    let radius =
                        location.radius_meters.unwrap_or(0.) + book.config.fix_error_meters;
                    for plane in 0..3 {
                        for i in 0..12 {
                            let angle = f64::from(i) * std::f64::consts::TAU / 12.;
                            let mut p = origin;
                            p[plane] += radius * angle.cos();
                            p[(plane + 1) % 3] += radius * angle.sin();
                            out.push(p);
                        }
                    }
                    out
                };
                let axes = self.ship.snapshot().pose.axes();
                let ship = self
                    .ship
                    .snapshot()
                    .pose
                    .position_meters
                    .map(|v| (v / book.config.grid_meters).round() * book.config.grid_meters);
                for (i, p) in positions.iter().enumerate() {
                    let relative = salimon_math::sub(*p, ship);
                    range = range.max(salimon_math::length(relative));
                    contacts.push(salimon_renderer::HologramContact {
                        relative_ship_meters: axes.map(|a| salimon_math::dot(relative, a)),
                        selected: i == 0 && mark.bearing.is_none(),
                        state: salimon_renderer::HologramContactState::Lead,
                    });
                }
            }
        }
        let frame = super::frames::character_ship_frame(pose);
        salimon_renderer::scanner_hologram(&contacts, self.ship.snapshot().power_available, range)
            .into_iter()
            .map(|mut i| {
                i.center_meters = frame.local_to_world(i.center_meters);
                i.orientation = pose.orientation.map(|v| v as f32);
                i
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::input::{self, Key};
    fn fixture() -> GameplaySession {
        let mut s = GameplaySession::default();
        crate::e2e::initialize(
            &mut s,
            crate::e2e::Config {
                scenario: crate::e2e::Scenario::PlanetaryChart,
                seed: 0,
                step: std::time::Duration::from_millis(16),
                fragment_load: None,
            },
        )
        .unwrap();
        s
    }
    #[test]
    fn surface_chart_unconstrained_heading_never_draws_a_narrow_or_inverted_cone() {
        let mut s = fixture();
        let now = super::super::field_marks::epoch(s.elapsed).unwrap();
        let b = s.field_marks.book.as_mut().unwrap();
        b.config.bearing_error_radians = std::f64::consts::PI;
        b.mark(Some([1., 0., 0.]), String::new(), now).unwrap();
        s.surface_chart.open = true;
        s.surface_chart.selected = 2;
        assert!(s.surface_hint().unwrap().contains("UNCONSTRAINED"));
        assert_eq!(
            s.surface_geometry(s.ship.snapshot().pose).len(),
            salimon_renderer::scanner_hologram(&[], true, 10_000.).len()
        );
    }
    #[test]
    fn surface_chart_live_editor_manual_target_deletion_and_reload_keep_world_and_history() {
        let mut s = fixture();
        let world = salimon_world::canonical::LANDMARKS.binding();
        let knowledge = s.field_capture().unwrap();
        input::dispatch(&mut s, Key::SurfaceChart, true);
        assert!(s.surface_chart.open);
        input::dispatch(&mut s, Key::Recovery, true);
        assert!(s.surface_chart.editing);
        s.surface_text("quiet valley").unwrap();
        input::dispatch(&mut s, Key::Confirm, true);
        assert_eq!(s.surface_chart.source.entries[&1].note(), "quiet valley");
        input::dispatch(&mut s, Key::Confirm, true);
        assert_eq!(s.surface_chart.source.target, Some(1));
        let controls = s.ship.snapshot();
        input::dispatch(&mut s, Key::Forward, true);
        assert_eq!(
            s.ship.snapshot().thruster_percentage,
            controls.thruster_percentage
        );
        input::dispatch(&mut s, Key::ScannerNext, true);
        let count = s.surface_chart.source.targets.len();
        input::dispatch(&mut s, Key::Confirm, true);
        assert_eq!(s.surface_chart.source.targets.len(), count);
        input::dispatch(&mut s, Key::Delete, true);
        assert!(s.surface_chart.source.entries[&2].deleted());
        assert_eq!(s.field_capture().unwrap(), knowledge);
        assert_eq!(salimon_world::canonical::LANDMARKS.binding(), world);
        let bytes = s.save_bytes(100).unwrap();
        let mut r = GameplaySession::default();
        r.restore_bytes(&bytes).unwrap();
        assert!(!r.surface_chart.open && !r.surface_chart.editing);
        assert_eq!(r.save_bytes(100).unwrap(), bytes);
        r.scanner.advance_closed(10_800_001).unwrap();
        assert_eq!(r.surface_capture().unwrap().unwrap().entries.len(), 2);
        assert_eq!(r.surface_chart.source.entries[&1].note(), "quiet valley");
    }
    #[test]
    fn surface_chart_power_modal_edit_validation_and_current_omission_reject_safely() {
        let mut s = fixture();
        input::dispatch(&mut s, Key::SurfaceChart, true);
        s.ship.set_stored_core_energy_joules(0);
        input::dispatch(&mut s, Key::Recovery, true);
        assert!(!s.surface_chart.editing);
        assert!(s.surface_hint().unwrap().contains("POWER OFF"));
        let source = s.surface_capture().unwrap();
        assert!(s.surface_text("bad").is_err());
        assert_eq!(s.surface_capture().unwrap(), source);
        let bytes = s.save_bytes(100).unwrap();
        let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        value["state"]
            .as_object_mut()
            .unwrap()
            .remove("surface_chart");
        let before = s.save_bytes(100).unwrap();
        assert!(
            s.restore_bytes(&serde_json::to_vec(&value).unwrap())
                .is_err()
        );
        assert!(s.save_bytes(100).unwrap() == before);
        value["schema_version"] = 27.into();
        let mut legacy = GameplaySession::default();
        legacy
            .restore_bytes(&serde_json::to_vec(&value).unwrap())
            .unwrap();
        assert_eq!(legacy.surface_capture().unwrap().unwrap().entries.len(), 2);
    }
    #[test]
    fn surface_chart_real_published_terrain_site_can_supply_a_close_inspection_receipt() {
        use salimon_world::{
            environment::landmarks::{Geography, Landmarks, Scale, Signature},
            terrain::{Feature, Layer, LayerKind, TerrainDefinition},
        };
        let body = salimon_world::CELESTIAL_BODIES[4];
        let g = TerrainDefinition::new(
            body,
            1,
            91,
            vec![Layer {
                kind: LayerKind::Ridges,
                amplitude_meters: 400.,
                wavelength_meters: 2000.,
            }],
            vec![Feature {
                direction: [0., 0., 1.],
                radius_meters: 400.,
                elevation_meters: -200.,
            }],
        )
        .unwrap()
        .geology();
        let landmarks = Landmarks::publish(
            g.clone(),
            Geography::Observed,
            0,
            &[Signature {
                key: "published-basin".into(),
                direction: [0., 0., 1.],
                scale: Scale::Local,
            }],
        )
        .unwrap();
        let site = &landmarks.sites()[0];
        let observer = salimon_math::add(site.position_meters, [0., 0., 4.]);
        let epoch = super::super::field_marks::epoch(std::time::Duration::ZERO).unwrap();
        let mut b = Book::new(
            salimon_universe::CatalogId::from_source("test", "physical-planet").unwrap(),
            "a".repeat(64),
            epoch,
        )
        .unwrap();
        b.fix(observer, epoch).unwrap();
        b.mark(
            Some(salimon_math::sub(site.position_meters, observer)),
            "source sighting".into(),
            epoch,
        )
        .unwrap();
        let original = g.to_bytes();
        let mut chart = Chart::default();
        chart
            .inspect(
                &b,
                1,
                InspectionSample {
                    site_id: site.id.clone(),
                    publication: landmarks.identity().into(),
                    kind: "geological".into(),
                    site: site.position_meters,
                    observer,
                    epoch,
                },
            )
            .unwrap();
        assert_eq!(chart.entries[&1].inspected().unwrap().site_id, site.id);
        assert_eq!(g.to_bytes(), original);
        landmarks.verify_binding(&landmarks.binding()).unwrap();
    }
}
