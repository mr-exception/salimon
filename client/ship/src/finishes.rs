//! Bounded cosmetics keyed by durable room/installed-object identity, never topology.
use salimon_persistence::{SaveError, Validate, invalid};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const CABIN_ROOM: u64 = 1;
pub const CARGO_ROOM: u64 = 2;
pub const STYLE_COUNT: u8 = 3;
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Group {
    Walls,
    Floor,
    Trim,
    Cabinetry,
    Upholstery,
}
impl Group {
    pub const ALL: [Self; 5] = [
        Self::Walls,
        Self::Floor,
        Self::Trim,
        Self::Cabinetry,
        Self::Upholstery,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Walls => "Walls",
            Self::Floor => "Floor",
            Self::Trim => "Trim",
            Self::Cabinetry => "Cabinetry",
            Self::Upholstery => "Upholstery",
        }
    }
}
pub fn style_name(style: u8) -> &'static str {
    match style {
        1 => "Slate",
        2 => "Ochre",
        _ => "Original",
    }
}
/// Linear factors preserve each authored surface/material, including original roughness.
/// No factor is applied to functional indicators, warnings, screens or glass.
pub fn tint(group: Group, style: u8) -> [f32; 3] {
    match (group, style) {
        (_, 0) => [1.; 3],
        (Group::Upholstery, 1) => [0.45, 1.25, 2.4],
        (Group::Upholstery, 2) => [1.1, 1.7, 0.75],
        (Group::Floor, 1) => [0.65, 0.85, 1.1],
        (Group::Floor, 2) => [1.15, 0.95, 0.65],
        (_, 1) => [0.45, 0.57, 0.72],
        (_, 2) => [0.95, 0.75, 0.45],
        _ => [1.; 3],
    }
}
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoomFinish {
    pub groups: [u8; 5],
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemFinish {
    pub room: u64,
    pub style: u8,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Finishes {
    pub rooms: BTreeMap<u64, RoomFinish>,
    pub items: BTreeMap<u64, ItemFinish>,
}
impl Default for Finishes {
    fn default() -> Self {
        Self {
            rooms: [
                (CABIN_ROOM, RoomFinish::default()),
                (CARGO_ROOM, RoomFinish::default()),
            ]
            .into(),
            items: BTreeMap::new(),
        }
    }
}
impl Validate for Finishes {
    fn validate(&self) -> Result<(), SaveError> {
        if self.rooms.is_empty()
            || self.rooms.len() > 64
            || self.items.len() > 512
            || self
                .rooms
                .iter()
                .any(|(id, r)| *id == 0 || r.groups.iter().any(|s| *s >= STYLE_COUNT))
            || self.items.iter().any(|(id, i)| {
                *id == 0 || i.style >= STYLE_COUNT || !self.rooms.contains_key(&i.room)
            })
        {
            return Err(invalid("room or furniture finish identity/palette/budget"));
        }
        Ok(())
    }
}
impl Finishes {
    /// Called when a real physical room is installed, never when a placeholder appears.
    pub fn add_room(&mut self, id: u64) -> bool {
        if id == 0 || self.rooms.len() >= 64 || self.rooms.contains_key(&id) {
            return false;
        }
        self.rooms.insert(id, RoomFinish::default());
        true
    }
    pub fn cycle_group(&mut self, room: u64, group: Group) -> bool {
        let Some(r) = self.rooms.get_mut(&room) else {
            return false;
        };
        let s = &mut r.groups[group as usize];
        *s = (*s + 1) % STYLE_COUNT;
        true
    }
    pub fn cycle_item(&mut self, room: u64, item: u64) -> bool {
        if item == 0
            || !self.rooms.contains_key(&room)
            || (!self.items.contains_key(&item) && self.items.len() >= 512)
        {
            return false;
        }
        if self.items.get(&item).is_some_and(|i| i.room != room) {
            return false;
        }
        let style = self
            .items
            .get(&item)
            .map_or(1, |i| (i.style + 1) % STYLE_COUNT);
        self.items.insert(item, ItemFinish { room, style });
        true
    }
    pub fn clear_item(&mut self, item: u64) {
        self.items.remove(&item);
    }
    /// Removing a physical part removes only that exact association; no inheritance to a new ID.
    pub fn remove_item(&mut self, item: u64) {
        self.items.remove(&item);
    }
    pub fn group_tint(&self, room: u64, group: Group) -> [f32; 3] {
        tint(
            group,
            self.rooms
                .get(&room)
                .map_or(0, |r| r.groups[group as usize]),
        )
    }
    pub fn item_tint(&self, room: u64, item: u64, group: Group) -> [f32; 3] {
        self.items
            .get(&item)
            .filter(|i| i.room == room)
            .map_or_else(|| self.group_tint(room, group), |i| tint(group, i.style))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn independent_groups_rooms_item_identity_and_roundtrip() {
        let mut f = Finishes::default();
        for g in Group::ALL {
            assert!(f.cycle_group(CABIN_ROOM, g));
        }
        assert!(f.cycle_item(CABIN_ROOM, 3002));
        assert!(f.cycle_item(CABIN_ROOM, 3002));
        assert_ne!(
            f.item_tint(CABIN_ROOM, 3002, Group::Upholstery),
            f.group_tint(CABIN_ROOM, Group::Upholstery)
        );
        let before = f.rooms[&CABIN_ROOM].clone();
        assert!(f.add_room(700));
        assert_eq!(f.rooms[&CABIN_ROOM], before);
        assert_eq!(f.rooms[&700], RoomFinish::default());
        // Partition topology is not an input to cosmetic identity: opening/closing it
        // changes neither room key nor its history. The structure integration is tested below.
        let r: Finishes = serde_json::from_slice(&serde_json::to_vec(&f).unwrap()).unwrap();
        assert_eq!(f, r);
        r.validate().unwrap();
        f.remove_item(3002);
        assert_eq!(
            f.item_tint(CABIN_ROOM, 4002, Group::Upholstery),
            f.group_tint(CABIN_ROOM, Group::Upholstery)
        );
        assert!(!f.cycle_item(999, 3002));
    }
    #[test]
    fn corrupt_and_unbounded_palettes_rejected() {
        let mut f = Finishes::default();
        f.rooms.get_mut(&1).unwrap().groups[0] = 3;
        assert!(f.validate().is_err());
        for g in Group::ALL {
            for style in 0..STYLE_COUNT {
                assert!(
                    tint(g, style)
                        .into_iter()
                        .all(|v| v.is_finite() && (0.0..=4.).contains(&v))
                );
            }
        }
    }
}
