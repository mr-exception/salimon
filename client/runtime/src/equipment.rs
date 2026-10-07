//! Portable toolbar state; presentation and individual tool behavior are separate.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ToolbarSlot {
    One,
    Two,
    Three,
    Four,
    Five,
}

impl ToolbarSlot {
    pub(crate) fn index(self) -> usize {
        match self {
            Self::One => 0,
            Self::Two => 1,
            Self::Three => 2,
            Self::Four => 3,
            Self::Five => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EquipmentTool {
    MiningTool,
}

pub(crate) struct EquipmentToolbar {
    slots: [Option<EquipmentTool>; 5],
    selected: Option<ToolbarSlot>,
}

impl Default for EquipmentToolbar {
    fn default() -> Self {
        Self {
            slots: [Some(EquipmentTool::MiningTool), None, None, None, None],
            selected: None,
        }
    }
}

impl EquipmentToolbar {
    pub(crate) fn mining_equipped(&self) -> bool {
        self.selected.and_then(|slot| self.slots[slot.index()]) == Some(EquipmentTool::MiningTool)
    }

    pub(crate) fn slots(&self) -> &[Option<EquipmentTool>; 5] {
        &self.slots
    }

    pub(crate) fn selected(&self) -> Option<ToolbarSlot> {
        self.selected
    }

    pub(crate) fn presentation(&self) -> salimon_renderer::EquipmentToolbar {
        use salimon_renderer::{EquipmentIcon, EquipmentSlot};
        salimon_renderer::EquipmentToolbar {
            slots: self.slots.map(|tool| {
                tool.map(|tool| match tool {
                    EquipmentTool::MiningTool => EquipmentIcon::MiningTool,
                })
            }),
            selected: self.selected.map(|slot| match slot {
                ToolbarSlot::One => EquipmentSlot::One,
                ToolbarSlot::Two => EquipmentSlot::Two,
                ToolbarSlot::Three => EquipmentSlot::Three,
                ToolbarSlot::Four => EquipmentSlot::Four,
                ToolbarSlot::Five => EquipmentSlot::Five,
            }),
        }
    }

    /// World session remains the authority for whether a physical object is held.
    pub(crate) fn select(&mut self, slot: ToolbarSlot, carrying: bool) {
        self.sync_carrying(carrying);
        if !carrying {
            self.selected = Some(slot);
        }
    }

    pub(crate) fn sync_carrying(&mut self, carrying: bool) {
        if carrying {
            self.selected = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_loadout_and_empty_slot_selection() {
        let mut toolbar = EquipmentToolbar::default();
        assert_eq!(
            toolbar.slots(),
            &[Some(EquipmentTool::MiningTool), None, None, None, None]
        );
        assert_eq!(toolbar.selected(), None);
        for slot in [
            ToolbarSlot::One,
            ToolbarSlot::Two,
            ToolbarSlot::Three,
            ToolbarSlot::Four,
            ToolbarSlot::Five,
        ] {
            toolbar.select(slot, false);
            assert_eq!(toolbar.selected(), Some(slot));
            toolbar.select(slot, false);
            assert_eq!(toolbar.selected(), Some(slot), "selection is not a toggle");
        }
    }

    #[test]
    fn presentation_maps_contents_and_selection_without_equipping_tools() {
        let mut toolbar = EquipmentToolbar::default();
        let initial = toolbar.presentation();
        assert_eq!(
            initial.slots,
            [
                Some(salimon_renderer::EquipmentIcon::MiningTool),
                None,
                None,
                None,
                None
            ]
        );
        assert_eq!(initial.selected, None);
        for (slot, expected) in [
            (ToolbarSlot::One, salimon_renderer::EquipmentSlot::One),
            (ToolbarSlot::Two, salimon_renderer::EquipmentSlot::Two),
            (ToolbarSlot::Three, salimon_renderer::EquipmentSlot::Three),
            (ToolbarSlot::Four, salimon_renderer::EquipmentSlot::Four),
            (ToolbarSlot::Five, salimon_renderer::EquipmentSlot::Five),
        ] {
            toolbar.select(slot, false);
            assert_eq!(toolbar.presentation().selected, Some(expected));
            assert_eq!(toolbar.presentation().slots, initial.slots);
        }
        toolbar.sync_carrying(true);
        assert_eq!(toolbar.presentation().selected, None);
    }

    #[test]
    fn carrying_clears_and_locks_selection_without_restore() {
        let mut toolbar = EquipmentToolbar::default();
        toolbar.select(ToolbarSlot::One, false);
        toolbar.sync_carrying(true);
        assert_eq!(toolbar.selected(), None);
        for slot in [
            ToolbarSlot::One,
            ToolbarSlot::Two,
            ToolbarSlot::Three,
            ToolbarSlot::Four,
            ToolbarSlot::Five,
        ] {
            toolbar.select(slot, true);
            assert_eq!(toolbar.selected(), None);
        }
        toolbar.sync_carrying(false);
        assert_eq!(toolbar.selected(), None);
        toolbar.select(ToolbarSlot::Five, false);
        assert_eq!(toolbar.selected(), Some(ToolbarSlot::Five));
    }
}
