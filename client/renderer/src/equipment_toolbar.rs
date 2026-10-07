//! Renderer-owned toolbar rasterization; selection and contents are supplied by runtime.

use crate::{OverlayImage, OverlayPlacement};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EquipmentIcon {
    MiningTool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EquipmentSlot {
    One,
    Two,
    Three,
    Four,
    Five,
}

impl EquipmentSlot {
    fn index(self) -> usize {
        match self {
            Self::One => 0,
            Self::Two => 1,
            Self::Three => 2,
            Self::Four => 3,
            Self::Five => 4,
        }
    }
}

/// Presentation snapshot, with no tool usability or input policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EquipmentToolbar {
    pub slots: [Option<EquipmentIcon>; 5],
    pub selected: Option<EquipmentSlot>,
}

const SLOT: u32 = 34;
const GAP: u32 = 4;
const WIDTH: u32 = 5 * SLOT + 4 * GAP;
const BACKGROUND: [u8; 4] = [8, 21, 29, 224];
const BORDER: [u8; 4] = [67, 95, 108, 255];
const SELECTED: [u8; 4] = [89, 224, 215, 255];
const INK: [u8; 4] = [220, 233, 236, 255];

#[derive(Default)]
pub(crate) struct ToolbarRaster {
    key: Option<(EquipmentToolbar, u32)>,
    pixels: Vec<u8>,
    revision: u64,
}

impl ToolbarRaster {
    pub(crate) fn update(&mut self, state: EquipmentToolbar, scale_factor: f64) {
        // Match existing HUD density handling: finite DPI, bounded to 1x–2x.
        let factor = if scale_factor.is_finite() {
            scale_factor
        } else {
            1.0
        };
        let scale = (factor.clamp(1.0, 2.0) * 2.0).round() as u32;
        if self.key == Some((state, scale)) {
            return;
        }
        self.key = Some((state, scale));
        self.pixels = vec![0; (WIDTH * SLOT * scale * scale * 4) as usize];
        self.revision = self.revision.wrapping_add(1);
        for (index, icon) in state.slots.iter().enumerate() {
            let x = index as u32 * (SLOT + GAP);
            let selected = state.selected.map(EquipmentSlot::index) == Some(index);
            self.rect(x, 0, SLOT, SLOT, if selected { SELECTED } else { BORDER });
            self.rect(
                x + 1,
                1,
                SLOT - 2,
                SLOT - 2,
                if selected {
                    [16, 49, 56, 240]
                } else {
                    BACKGROUND
                },
            );
            for (row, bits) in
                crate::cockpit_instruments::glyph_rows(char::from(b'1' + index as u8))
                    .iter()
                    .enumerate()
            {
                for col in 0..5 {
                    if bits & (1 << (4 - col)) != 0 {
                        self.rect(x + 4 + col, 4 + row as u32, 1, 1, INK);
                    }
                }
            }
            if *icon == Some(EquipmentIcon::MiningTool) {
                // Compact side-profile drill: grip, housing, cyan status stripe, bit.
                self.rect(x + 10, 23, 5, 7, INK);
                self.rect(x + 9, 16, 16, 8, INK);
                self.rect(x + 11, 18, 10, 4, [35, 70, 81, 255]);
                self.rect(x + 12, 18, 3, 4, SELECTED);
                self.rect(x + 25, 18, 4, 4, INK);
                self.rect(x + 29, 19, 2, 2, INK);
            }
        }
    }

    fn rect(&mut self, x: u32, y: u32, width: u32, height: u32, color: [u8; 4]) {
        let scale = self.key.expect("raster key is installed before drawing").1;
        for py in y * scale..(y + height) * scale {
            for px in x * scale..(x + width) * scale {
                let offset = ((py * WIDTH * scale + px) * 4) as usize;
                self.pixels[offset..offset + 4].copy_from_slice(&color);
            }
        }
    }

    pub(crate) fn image(&self) -> Option<OverlayImage<'_>> {
        let scale = self.key?.1;
        Some(OverlayImage {
            width: WIDTH * scale,
            height: SLOT * scale,
            rgba8: &self.pixels,
            revision: self.revision,
            placement: OverlayPlacement::BottomCenter,
        })
    }

    /// Reserve the fitted toolbar height plus a gap for screen-space messages.
    pub(crate) fn message_inset(&self, surface: [u32; 2]) -> u32 {
        self.image().map_or(0, |image| {
            let size = crate::overlay::fitted_overlay_size(surface, [image.width, image.height]);
            (size[1] + 8.0 * self.key.expect("image requires raster key").1 as f32).ceil() as u32
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(selected: Option<EquipmentSlot>) -> EquipmentToolbar {
        EquipmentToolbar {
            slots: [Some(EquipmentIcon::MiningTool), None, None, None, None],
            selected,
        }
    }

    #[test]
    fn raster_has_five_numbered_slots_icon_empty_centers_and_single_highlight() {
        let mut raster = ToolbarRaster::default();
        for selection in [
            None,
            Some(EquipmentSlot::One),
            Some(EquipmentSlot::Two),
            Some(EquipmentSlot::Five),
        ] {
            raster.update(state(selection), 1.0);
            let image = raster.image().unwrap();
            assert_eq!([image.width, image.height], [372, 68]);
            image.validate().unwrap();
            for index in 0..5 {
                let pixel = |x: u32, y: u32| {
                    let offset = ((y * 2 * image.width + x * 2) * 4) as usize;
                    &image.rgba8[offset..offset + 4]
                };
                let x = index * (SLOT + GAP);
                assert_eq!(
                    pixel(x, 0),
                    if selection.map(EquipmentSlot::index) == Some(index as usize) {
                        &SELECTED
                    } else {
                        &BORDER
                    }
                );
                assert_eq!(
                    pixel(x + 10, 16),
                    if index == 0 {
                        &INK
                    } else if selection.map(EquipmentSlot::index) == Some(index as usize) {
                        &[16, 49, 56, 240]
                    } else {
                        &BACKGROUND
                    }
                );
                // Each number contains visible ink; the inter-slot gap stays transparent.
                assert!((4..11).any(|y| (4..9).any(|dx| pixel(x + dx, y) == INK)));
                if index < 4 {
                    assert_eq!(pixel(x + SLOT, 0), &[0; 4]);
                }
            }
        }
    }

    #[test]
    fn cache_and_dpi_follow_display_state_without_gameplay_policy() {
        let mut raster = ToolbarRaster::default();
        raster.update(state(None), 1.0);
        let revision = raster.image().unwrap().revision;
        raster.update(state(None), f64::NAN);
        assert_eq!(raster.image().unwrap().revision, revision);
        raster.update(state(Some(EquipmentSlot::Three)), 1.0);
        assert_ne!(raster.image().unwrap().revision, revision);
        raster.update(state(None), 2.0);
        assert_eq!(raster.image().unwrap().height, 136);
        for surface in [[1280, 720], [360, 640], [1920, 1080]] {
            let image = raster.image().unwrap();
            let size = crate::overlay::fitted_overlay_size(surface, [image.width, image.height]);
            assert!(raster.message_inset(surface) as f32 > size[1]);
        }
        // Contents come from the DTO even if all five slots are empty.
        raster.update(
            EquipmentToolbar {
                slots: [None; 5],
                selected: None,
            },
            1.0,
        );
        assert!(raster.image().is_some());
    }
}
