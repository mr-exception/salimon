//! Durable preferences keyed by physical room identity. Bus/damage supersede them.
use salimon_persistence::{SaveError, Validate, invalid};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Preset {
    #[default]
    Work,
    Relax,
    Night,
    Custom,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Setting {
    pub enabled: bool,
    pub brightness: u8,
    pub warmth: u8,
}
impl Default for Setting {
    fn default() -> Self {
        Self {
            enabled: true,
            brightness: 70,
            warmth: 70,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Room {
    pub operable: [bool; 5],
    pub preset: Preset,
    pub master: bool,
    pub general: Setting,
    pub fixtures: [Option<Setting>; 5],
}
impl Default for Room {
    fn default() -> Self {
        Self {
            operable: [true; 5],
            preset: Preset::Work,
            master: true,
            general: Default::default(),
            fixtures: [None; 5],
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lighting {
    pub rooms: BTreeMap<u64, Room>,
}
impl Default for Lighting {
    fn default() -> Self {
        Self {
            rooms: [(1, Room::default()), (2, Room::default())].into(),
        }
    }
}
impl Validate for Lighting {
    fn validate(&self) -> Result<(), SaveError> {
        if self.rooms.is_empty()
            || self.rooms.len() > 64
            || self.rooms.keys().any(|id| *id == 0)
            || self.rooms.values().any(|r| {
                std::iter::once(&r.general)
                    .chain(r.fixtures.iter().flatten())
                    .any(|s| s.brightness > 100 || s.warmth > 100)
            })
        {
            return Err(invalid("room light identity/setting budget"));
        }
        Ok(())
    }
}
impl Room {
    pub fn next_preset(&mut self) {
        self.preset = match self.preset {
            Preset::Work => Preset::Relax,
            Preset::Relax => Preset::Night,
            _ => Preset::Work,
        };
        let (brightness, warmth) = match self.preset {
            Preset::Relax => (40, 90),
            Preset::Night => (8, 100),
            _ => (70, 70),
        };
        self.general = Setting {
            enabled: true,
            brightness,
            warmth,
        };
        self.fixtures = [None; 5];
    }
    pub fn effective(&self, index: usize, powered: bool, operable: bool) -> Setting {
        let mut s = self.fixtures[index].unwrap_or(self.general);
        s.enabled &= self.master && powered && operable && self.operable[index];
        s
    }
}
pub fn rgb(s: Setting) -> [f32; 3] {
    let w = f32::from(s.warmth) / 100.;
    let b = if s.enabled {
        f32::from(s.brightness) / 100.
    } else {
        0.
    };
    [b, b * (0.94 - 0.28 * w), b * (1. - 0.55 * w)]
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn outage_and_damage_preserve_preferences_and_master_supersedes_override() {
        let mut r = Room::default();
        r.next_preset();
        r.fixtures[0] = Some(Setting {
            brightness: 100,
            ..Default::default()
        });
        let saved = r.clone();
        assert!(!r.effective(0, false, true).enabled);
        assert!(!r.effective(0, true, false).enabled);
        assert_eq!(r, saved);
        assert_eq!(r.effective(0, true, true).brightness, 100);
        r.master = false;
        assert!(!r.effective(0, true, true).enabled);
    }
    #[test]
    fn independent_room_histories_and_malformed_settings() {
        let mut s = Lighting::default();
        s.rooms.get_mut(&1).unwrap().next_preset();
        s.rooms.insert(3, Room::default());
        assert_ne!(s.rooms[&1], s.rooms[&2]);
        let r: Lighting = serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
        assert_eq!(r, s);
        s.rooms.get_mut(&3).unwrap().general.warmth = 101;
        assert!(s.validate().is_err());
    }
}
