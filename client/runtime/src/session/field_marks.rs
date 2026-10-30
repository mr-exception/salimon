//! Suit-local measurement composition. Never queries catalog points of interest.
use super::GameplaySession;
use salimon_persistence::{SaveError, invalid};
use salimon_universe::{Epoch, TimeScale, surface_observations::Book};
use std::time::Duration;
#[derive(Default)]
pub(crate) struct State {
    pub(crate) book: Option<Book>,
    last_sample: Option<[f64; 3]>,
    show_until: Duration,
    last_update: Duration,
}
pub(crate) fn epoch(elapsed: Duration) -> Result<Epoch, SaveError> {
    let nanos = i64::try_from(elapsed.as_nanos()).map_err(|_| invalid("suit clock range"))?;
    Epoch::calendar([2000, 1, 1], [12, 0, 0, 0], TimeScale::Tt)
        .and_then(|e| e.advance_nanos(nanos))
        .map_err(|e| invalid(e.to_string()))
}
fn new_book(elapsed: Duration) -> Result<Book, SaveError> {
    let binding = super::canonical::definition_binding();
    let mut book = Book::new(binding.body, binding.sha256, epoch(elapsed)?)
        .map_err(|e| invalid(e.to_string()))?;
    book.config = serde_json::from_str(include_str!("../../config/suit_navigation.json"))
        .map_err(|e| invalid(e.to_string()))?;
    book.validate().map_err(|e| invalid(e.to_string()))?;
    Ok(book)
}
pub(crate) fn validate_book(book: &Book, elapsed: Duration) -> Result<(), SaveError> {
    let binding = super::canonical::definition_binding();
    book.compatible(&binding.body, &binding.sha256)
        .map_err(|e| SaveError::IncompatibleCatalog(e.to_string()))?;
    let now = epoch(elapsed)?;
    if now
        .seconds_since(book.estimate.measured_at)
        .map_err(|e| invalid(e.to_string()))?
        < 0.
        || book
            .marks()
            .iter()
            .any(|m| now.seconds_since(m.epoch).map_or(true, |s| s < 0.))
    {
        return Err(invalid("field observation ahead of shared clock"));
    }
    Ok(())
}
pub(crate) fn restore(
    book: Option<Book>,
    schema: u32,
    canonical: bool,
    elapsed: Duration,
) -> Result<State, SaveError> {
    let book = match (canonical, schema, book) {
        (true, _, Some(b)) => {
            validate_book(&b, elapsed)?;
            Some(b)
        }
        (true, 1..=26, None) => Some(new_book(elapsed)?),
        (false, _, None) => None,
        _ => {
            return Err(invalid(
                "field observations missing/incompatible with world",
            ));
        }
    };
    Ok(State {
        book,
        ..Default::default()
    })
}
impl GameplaySession {
    pub(crate) fn field_capture(&self) -> Result<Option<Book>, SaveError> {
        if self.mining.terrain != salimon_world::terrain::TerrainMode::Canonical {
            return Ok(None);
        }
        let book = match &self.field_marks.book {
            Some(b) => b.clone(),
            None => new_book(self.elapsed)?,
        };
        validate_book(&book, self.elapsed)?;
        Ok(Some(book))
    }
    pub(crate) fn reset_field_sample(&mut self) {
        self.field_marks.last_sample = Some(self.snapshot().character.eye_position_meters);
    }
    pub(crate) fn update_field_tick(&mut self) {
        if self.mining.terrain != salimon_world::terrain::TerrainMode::Canonical {
            return;
        }
        let result = (|| -> Result<(), SaveError> {
            let p = self.snapshot().character.eye_position_meters;
            let delta = self
                .field_marks
                .last_sample
                .map_or([0.; 3], |last| salimon_math::sub(p, last));
            let now = epoch(self.elapsed)?;
            if self.field_marks.book.is_none() {
                self.field_marks.book = Some(new_book(self.elapsed)?);
            }
            self.field_marks
                .book
                .as_mut()
                .expect("initialized source")
                .drift(delta, now)
                .map_err(|e| invalid(e.to_string()))?;
            self.field_marks.last_sample = Some(p);
            if self.elapsed.saturating_sub(self.field_marks.last_update) >= Duration::from_secs(1) {
                self.update_field_measurement()?;
            }
            Ok(())
        })();
        if let Err(e) = result {
            log::warn!("suit measurement: {e}");
        }
    }
    pub(crate) fn update_field_measurement(&mut self) -> Result<(), SaveError> {
        if self.mining.terrain != salimon_world::terrain::TerrainMode::Canonical {
            return Ok(());
        }
        let snapshot = self.snapshot();
        let p = snapshot.character.eye_position_meters;
        let frame = super::frames::character_ship_frame(snapshot.ship.pose);
        // External roof beacon; the mount is outside the authored hull envelope.
        let beacon = frame.local_to_world([0., 4.5, 0.]);
        let ray = salimon_world::mining::MiningRay::new(
            salimon_world::WorldPosition::new(p[0], p[1], p[2]),
            salimon_world::WorldPosition::new(beacon[0], beacon[1], beacon[2]),
        );
        let distance = salimon_math::length(salimon_math::sub(beacon, p));
        let clear = ray.is_some_and(|ray| {
            salimon_world::mining::terrain_obstruction(ray, self.mining.terrain, distance)
                .is_none_or(|d| d >= distance - 0.05)
        }) && salimon_character::ship_sight_obstruction(
            frame.world_to_local(p),
            frame.world_to_local(beacon),
            true,
        )
        .is_none_or(|d| d >= distance - 0.05)
            && self.construction_solids().iter().all(|b| {
                salimon_character::cargo::box_sight_obstruction(
                    frame.world_to_local(p),
                    frame.world_to_local(beacon),
                    *b,
                )
                .is_none_or(|d| d >= distance - 0.05)
            });
        let clear = clear
            && self
                .organism_field()
                .sweep(p, beacon, [0.; 3])
                .is_ok_and(|hit| hit.is_none());
        let mut book = self.field_capture()?.expect("canonical field source");
        let now = epoch(self.elapsed)?;
        if book.enabled
            && snapshot.ship.power_available
            && clear
            && distance <= book.config.beacon_range_meters
        {
            book.fix(p, now).map_err(|e| invalid(e.to_string()))?;
        } else {
            let delta = self
                .field_marks
                .last_sample
                .map_or([0.; 3], |last| salimon_math::sub(p, last));
            book.drift(delta, now).map_err(|e| invalid(e.to_string()))?;
        }
        self.field_marks.last_update = self.elapsed;
        self.field_marks.last_sample = Some(p);
        self.field_marks.book = Some(book);
        Ok(())
    }
    pub(crate) fn field_action(&mut self, action: u8) {
        if self.paused
            || self.survival.snapshot().terminal
            || self.view_mode != super::ViewMode::Gameplay
            || self.wheel.open
            || self.crafting_ui.open
            || self.archive_reader.open
            || self.surface_chart.open
            || self.building.active
            || self.mining.terrain != salimon_world::terrain::TerrainMode::Canonical
        {
            return;
        }
        if self.update_field_measurement().is_err() {
            self.feedback = Some("SUIT NAVIGATION UNAVAILABLE");
            return;
        }
        let now = match epoch(self.elapsed) {
            Ok(e) => e,
            Err(_) => return,
        };
        let snapshot = self.snapshot();
        let book = self
            .field_marks
            .book
            .as_mut()
            .expect("canonical measurement initialized");
        if action == 2 {
            book.enabled = !book.enabled;
            if !book.enabled {
                let _ = book.drift([0.; 3], now);
            }
        } else {
            if action == 1 && (!book.enabled || book.estimate.position_meters.is_none()) {
                self.feedback = Some("BEARING UNAVAILABLE - ACQUIRE A NAV FIX");
                return;
            }
            let bearing = (action == 1).then(|| {
                salimon_math::sub(
                    snapshot.character.look_target_meters,
                    snapshot.character.eye_position_meters,
                )
            });
            if book.mark(bearing, String::new(), now).is_err() {
                self.feedback = Some("FIELD MARK FAILED - CAPACITY OR SENSOR DATA");
                return;
            }
        }
        self.field_marks.show_until = self.elapsed.saturating_add(Duration::from_secs(12));
    }
    pub(crate) fn field_hint(&self) -> Option<String> {
        if self.elapsed >= self.field_marks.show_until
            || self.view_mode != super::ViewMode::Gameplay
        {
            return None;
        }
        let b = self.field_marks.book.as_ref()?;
        let error = b
            .estimate
            .radius_meters
            .map_or("NO FIX".into(), |r| format!("+/-{r:.1}m"));
        let last = b.marks().last().map_or("NO MARKS", |m| {
            if m.bearing.is_some() {
                "BEARING ONLY - RANGE UNKNOWN"
            } else {
                "ESTIMATED SPOT"
            }
        });
        Some(format!(
            "SUIT {} {} | {} | MARKS {}\nF10 SPOT | F11 BEARING | F12 NAV {}",
            b.estimate.quality(),
            error,
            last,
            b.marks().len(),
            if b.enabled { "ON" } else { "OFF" }
        ))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{
        Command,
        input::{self, Key},
    };
    #[test]
    fn field_native_controls_record_bearings_and_unavailable_marks_without_catalog_reveal() {
        let mut s = GameplaySession::new_canonical_run().unwrap();
        s.command(Command::Control(crate::session::ControlKey::Backward, true));
        s.advance_automation(Duration::from_secs(8));
        s.command(Command::Control(
            crate::session::ControlKey::Backward,
            false,
        ));
        let chart = s.chart.checkpoint();
        let scanner = s.scanner.contacts.checkpoint();
        input::dispatch(&mut s, Key::FieldSpot, true);
        input::dispatch(&mut s, Key::FieldBearing, true);
        let b = s.field_marks.book.as_ref().unwrap();
        assert_eq!(b.marks().len(), 2);
        assert!(b.marks()[1].bearing.is_some());
        input::dispatch(&mut s, Key::FieldPower, true);
        assert_eq!(
            s.field_marks.book.as_ref().unwrap().estimate.quality(),
            "UNAVAILABLE"
        );
        input::dispatch(&mut s, Key::FieldBearing, true);
        assert_eq!(s.field_marks.book.as_ref().unwrap().marks().len(), 2);
        input::dispatch(&mut s, Key::FieldSpot, true);
        assert_eq!(
            s.field_marks.book.as_ref().unwrap().marks()[2]
                .estimate
                .position_meters,
            None
        );
        assert_eq!(
            serde_json::to_value(s.chart.checkpoint()).unwrap(),
            serde_json::to_value(chart).unwrap()
        );
        assert_eq!(
            serde_json::to_value(s.scanner.contacts.checkpoint()).unwrap(),
            serde_json::to_value(scanner).unwrap()
        );
        let before = s.field_capture().unwrap();
        s.command(Command::TogglePause);
        input::dispatch(&mut s, Key::FieldSpot, true);
        s.command(Command::TogglePause);
        assert_eq!(s.field_capture().unwrap(), before);
        let before = s.save_bytes(100).unwrap();
        let mut r = GameplaySession::default();
        r.restore_bytes(&before).unwrap();
        assert_eq!(r.save_bytes(100).unwrap(), before);
    }
    #[test]
    fn field_restore_missing_current_source_bad_definition_and_future_epoch_are_atomic() {
        let s = GameplaySession::new_canonical_run().unwrap();
        let bytes = s.save_bytes(100).unwrap();
        for mutation in 0..3 {
            let mut v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            match mutation {
                0 => {
                    v["state"].as_object_mut().unwrap().remove("field_marks");
                }
                1 => v["state"]["field_marks"]["definition_sha256"] = "b".repeat(64).into(),
                _ => {
                    v["state"]["field_marks"]["estimate"]["measured_at"] =
                        serde_json::to_value(epoch(Duration::from_secs(1)).unwrap()).unwrap()
                }
            }
            let mut r = GameplaySession::new_canonical_run().unwrap();
            let before = r.save_bytes(100).unwrap();
            assert!(r.restore_bytes(&serde_json::to_vec(&v).unwrap()).is_err());
            assert_eq!(r.save_bytes(100).unwrap(), before);
        }
        let mut legacy: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        legacy["schema_version"] = 26.into();
        legacy["state"]
            .as_object_mut()
            .unwrap()
            .remove("field_marks");
        let mut r = GameplaySession::default();
        r.restore_bytes(&serde_json::to_vec(&legacy).unwrap())
            .unwrap();
        assert!(r.field_marks.book.unwrap().marks().is_empty());
    }
    #[test]
    fn field_beacon_requires_power_and_line_of_sight_and_fix_stays_estimated() {
        let mut s = GameplaySession::new_canonical_run().unwrap();
        s.command(Command::Control(crate::session::ControlKey::Backward, true));
        s.advance_automation(Duration::from_secs(8));
        s.command(Command::Control(
            crate::session::ControlKey::Backward,
            false,
        ));
        s.update_field_measurement().unwrap();
        let first = s.field_marks.book.as_ref().unwrap().estimate.clone();
        // Actual world geometry determines whether the roof beacon is visible.
        assert!(
            first.position_meters.is_some(),
            "visible roof beacon: {first:?}"
        );
        assert_eq!(first.radius_meters, Some(5.));
        s.ship.set_stored_core_energy_joules(0);
        s.update_field_measurement().unwrap();
        assert_ne!(
            s.field_marks.book.as_ref().unwrap().estimate.source,
            salimon_universe::surface_observations::Source::ShipBeacon
        );
    }
}
