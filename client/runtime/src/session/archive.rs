//! Local Archive source and modal reader. Recovery and charting are separate owners.
use super::{GameplaySession, ViewMode};
use salimon_persistence::{SaveError, Validate, invalid};

pub(crate) const TERMINAL: [f64; 3] = salimon_renderer::ARCHIVE_TERMINAL_METERS;
#[derive(Clone, Copy)]
struct Record {
    id: &'static str,
    title: &'static str,
    source: &'static str,
    status: &'static str,
    body: &'static str,
}
const RECORDS: [Record; 7] = [
    Record {
        id: "voyage.earth-departure",
        title: "Earth orbital departure",
        source: "Ship event log",
        status: "INTACT / RECORDED EVENT",
        body: "Departure from Earth orbit recorded.\nHistorical entry only. Present coordinates and route\nare unavailable.",
    },
    Record {
        id: "voyage.mars-transit",
        title: "Mars transit",
        source: "Ship event log",
        status: "INTACT / RECORDED EVENT",
        body: "Mars transit recorded.\nNo current navigational fix survives in this entry.",
    },
    Record {
        id: "routine.maintenance",
        title: "Routine maintenance",
        source: "Local maintenance log",
        status: "INTACT / LOCAL NOTE",
        body: "Cabin equipment checked. Loose supplies secured.\nRoutine maintenance entry; no destination attached.",
    },
    Record {
        id: "routine.personal",
        title: "A quiet day",
        source: "Local personal note",
        status: "INTACT / PERSONAL ACCOUNT",
        body: "A quiet day aboard. Time to put things in order.\nA personal note, not verified navigation data.",
    },
    Record {
        id: "voyage.system-exit",
        title: "Solar System exit",
        source: "Ship event log",
        status: "INTACT / RECORDED EVENT",
        body: "Leaving Solar System territory recorded.\nLast intact outbound boundary entry.\nThe subsequent journey is not established.",
    },
    Record {
        id: "received.damaged-01",
        title: "Damaged received record 01",
        source: "Received packet / sender unknown",
        status: "DAMAGED / CONTENT UNKNOWN",
        body: "Packet identity survives; content cannot be read.\nCorruption, encoding and missing segments must be\ndistinguished by the future Recovery Module.\nNo location or message claim has been recovered.",
    },
    Record {
        id: "received.damaged-02",
        title: "Damaged received record 02",
        source: "Received packet / sender unknown",
        status: "DAMAGED / CONTENT UNKNOWN",
        body: "Separate received packet; content cannot be read.\nIntegrity and chronology remain uncertain.\nRecovery hardware is not yet available.\nNo destination or historical event is confirmed.",
    },
];

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Archive {
    version: u32,
    selected: String,
    read: Vec<String>,
}
impl Default for Archive {
    fn default() -> Self {
        Self {
            version: 1,
            selected: RECORDS[0].id.into(),
            read: Vec::new(),
        }
    }
}
impl Validate for Archive {
    fn validate(&self) -> Result<(), SaveError> {
        let known = |id: &str| RECORDS.iter().any(|r| r.id == id);
        if self.version != 1
            || !known(&self.selected)
            || self.read.len() > RECORDS.len()
            || self
                .read
                .iter()
                .enumerate()
                .any(|(i, id)| !known(id) || self.read[..i].contains(id))
        {
            return Err(invalid("archive version or record identity"));
        }
        Ok(())
    }
}
#[derive(Default)]
pub(crate) struct Reader {
    pub(crate) open: bool,
}
#[derive(Clone, Copy)]
pub(crate) enum ReaderAction {
    Previous,
    Next,
    Close,
}

pub(crate) fn target(eye: [f64; 3], look: [f64; 3]) -> bool {
    let d = std::array::from_fn::<_, 3, _>(|i| look[i] - eye[i]);
    let t = (TERMINAL[2] - eye[2]) / d[2];
    let distance = eye
        .iter()
        .zip(TERMINAL)
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f64>()
        .sqrt();
    // Flush wall panel faces +Z; only cabin-side, close, forward ray hits count.
    t.is_finite()
        && t > 0.0
        && eye[2] > TERMINAL[2]
        && distance <= 2.5
        && (eye[0] + t * d[0] - TERMINAL[0]).abs() <= 0.4
        && (eye[1] + t * d[1] - TERMINAL[1]).abs() <= 0.3
        // The wall sight proxy is body-expanded by 0.24 m; the flush panel
        // sits inside that final hull skin. Earlier solids still block the ray.
        && salimon_character::ship_sight_obstruction(eye, TERMINAL, true)
            .is_none_or(|hit| hit >= distance - 0.25)
}
impl GameplaySession {
    pub(super) fn open_archive(&mut self) {
        if self.view_mode != ViewMode::Gameplay || !self.ship.snapshot().power_available {
            return;
        }
        self.clear_held_controls();
        self.archive_reader.open = true;
        self.validate_archive_focus();
        if self.archive_reader.open {
            self.mark_archive_read();
        }
    }
    fn mark_archive_read(&mut self) {
        if !self.archive.read.contains(&self.archive.selected) {
            self.archive.read.push(self.archive.selected.clone());
        }
    }
    pub(crate) fn archive_action(&mut self, action: ReaderAction) {
        self.validate_archive_focus();
        if self.is_paused() || !self.archive_reader.open {
            return;
        }
        let i = RECORDS
            .iter()
            .position(|r| r.id == self.archive.selected)
            .expect("validated archive selection");
        let next = match action {
            ReaderAction::Close => {
                self.archive_reader.open = false;
                return;
            }
            ReaderAction::Previous => i.saturating_sub(1),
            ReaderAction::Next => (i + 1).min(RECORDS.len() - 1),
        };
        self.archive.selected = RECORDS[next].id.into();
        self.mark_archive_read();
    }
    pub(super) fn validate_archive_focus(&mut self) {
        if !self.archive_reader.open {
            return;
        }
        let s = self.snapshot();
        let frame = super::frames::character_ship_frame(s.ship.pose);
        let eye = frame.world_to_local(s.character.eye_position_meters);
        if self.view_mode != ViewMode::Gameplay
            || self.survival.snapshot().terminal
            || !s.ship.power_available
            || s.character.location != salimon_character::CharacterLocation::InsideShip
            || !target(eye, TERMINAL)
        {
            self.archive_reader.open = false;
        }
    }
    pub(crate) fn archive_text(&self) -> Option<String> {
        if !self.archive_reader.open || self.is_paused() {
            return None;
        }
        let i = RECORDS.iter().position(|r| r.id == self.archive.selected)?;
        let r = RECORDS[i];
        Some(format!(
            "SCOUT ARCHIVE  {}/{}\n{}\n{}\nSOURCE: {}\nTIME: UNKNOWN / ORDER: RETAINED RECORD SEQUENCE\n\n{}\n\nREAD {}/{}  |  NO NAVIGATION FIX\nUP/DOWN: records  E: close  ESC: pause",
            i + 1,
            RECORDS.len(),
            r.title,
            r.status,
            r.source,
            r.body,
            self.archive.read.len(),
            RECORDS.len()
        ))
    }
}

#[cfg(test)]
mod tests;
