//! Renderer-owned toolbar rasterization; selection and contents are supplied by runtime.

use crate::{OverlayImage, OverlayPlacement};
use std::sync::OnceLock;

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
            if *icon == Some(EquipmentIcon::MiningTool) {
                self.mining_icon(x);
            }
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
        }
    }

    fn mining_icon(&mut self, slot_x: u32) {
        static ART: OnceLock<crate::item_image::ItemImage> = OnceLock::new();
        let art = ART.get_or_init(|| {
            crate::item_image::decode(include_bytes!("../../assets/items/mining-tool/icon.png"))
                .expect("checked-in mining icon PNG is covered by the artwork regression")
        });
        let scale = self.key.expect("raster key is installed before drawing").1;
        let bounds = alpha_bounds(art).expect("checked-in mining icon has visible artwork");
        let [x, y, width, height] = icon_placement(bounds, scale);
        for dy in 0..height {
            for dx in 0..width {
                let sx = bounds[0] + dx * (bounds[2] - bounds[0]) / width;
                let sy = bounds[1] + dy * (bounds[3] - bounds[1]) / height;
                let source = ((sy * art.width + sx) * 4) as usize;
                let pixel = &art.pixels[source..source + 4];
                if pixel[3] < 16 {
                    continue;
                }
                let target = (((y + dy) * WIDTH * scale + slot_x * scale + x + dx) * 4) as usize;
                let alpha = u32::from(pixel[3]);
                // Composite straight-alpha artwork onto the slot; do not replace
                // its background with the icon's asymmetric transparent canvas.
                for (channel, value) in pixel.iter().take(3).enumerate() {
                    self.pixels[target + channel] = ((u32::from(*value) * alpha
                        + u32::from(self.pixels[target + channel]) * (255 - alpha)
                        + 127)
                        / 255) as u8;
                }
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

// Half-open bounds of visible pixels, excluding antialias fringe below 16/255.
fn alpha_bounds(art: &crate::item_image::ItemImage) -> Option<[u32; 4]> {
    let mut bounds = [art.width, art.height, 0, 0];
    for y in 0..art.height {
        for x in 0..art.width {
            if art.pixels[((y * art.width + x) * 4 + 3) as usize] >= 16 {
                bounds[0] = bounds[0].min(x);
                bounds[1] = bounds[1].min(y);
                bounds[2] = bounds[2].max(x + 1);
                bounds[3] = bounds[3].max(y + 1);
            }
        }
    }
    (bounds[2] > bounds[0] && bounds[3] > bounds[1]).then_some(bounds)
}

fn icon_placement(bounds: [u32; 4], scale: u32) -> [u32; 4] {
    let width = bounds[2] - bounds[0];
    let height = bounds[3] - bounds[1];
    let available = (SLOT - 8) * scale;
    let longest = width.max(height);
    let fitted_width = (width * available / longest).max(1);
    let fitted_height = (height * available / longest).max(1);
    [
        (SLOT * scale - fitted_width) / 2,
        (SLOT * scale - fitted_height) / 2,
        fitted_width,
        fitted_height,
    ]
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
                if index == 0 {
                    assert!((12..26).any(|y| (4..30).any(|dx| {
                        let p = pixel(x + dx, y);
                        u16::from(p[0]) > u16::from(p[2]) + 20 // warm manufactured housing
                    })));
                } else {
                    assert_eq!(
                        pixel(x + 17, 17),
                        if selection.map(EquipmentSlot::index) == Some(index as usize) {
                            &[16, 49, 56, 240]
                        } else {
                            &BACKGROUND
                        }
                    );
                }
                // Each number contains visible ink; the inter-slot gap stays transparent.
                assert!((4..11).any(|y| (4..9).any(|dx| pixel(x + dx, y) == INK)));
                if index < 4 {
                    assert_eq!(pixel(x + SLOT, 0), &[0; 4]);
                }
            }
        }
    }

    #[test]
    fn artwork_is_centered_by_visible_bounds_despite_asymmetric_canvas() {
        let art =
            crate::item_image::decode(include_bytes!("../../assets/items/mining-tool/icon.png"))
                .unwrap();
        let bounds = alpha_bounds(&art).unwrap();
        // A padded, offset copy must receive identical fitted artwork placement.
        let mut padded = crate::item_image::ItemImage {
            width: art.width + 37,
            height: art.height + 21,
            pixels: vec![0; ((art.width + 37) * (art.height + 21) * 4) as usize],
        };
        for y in 0..art.height {
            for x in 0..art.width {
                let from = ((y * art.width + x) * 4) as usize;
                let to = (((y + 3) * padded.width + x + 29) * 4) as usize;
                padded.pixels[to..to + 4].copy_from_slice(&art.pixels[from..from + 4]);
            }
        }
        for scale in 2..=4 {
            let placement = icon_placement(bounds, scale);
            assert_eq!(
                placement,
                icon_placement(alpha_bounds(&padded).unwrap(), scale)
            );
            let [x, y, w, h] = placement;
            assert!((2 * x + w).abs_diff(SLOT * scale) <= 1);
            assert!((2 * y + h).abs_diff(SLOT * scale) <= 1);
            assert!(x >= 4 * scale && y >= 4 * scale);
            for selection in [None, Some(EquipmentSlot::One), Some(EquipmentSlot::Two)] {
                let mut raster = ToolbarRaster::default();
                raster.update(state(selection), f64::from(scale) / 2.0);
                let image = raster.image().unwrap();
                let mut empty = ToolbarRaster::default();
                empty.update(
                    EquipmentToolbar {
                        slots: [None; 5],
                        selected: selection,
                    },
                    f64::from(scale) / 2.0,
                );
                let background = empty.image().unwrap();
                let mut visible = [SLOT * scale, SLOT * scale, 0, 0];
                for py in 0..SLOT * scale {
                    for px in 0..SLOT * scale {
                        let offset = ((py * image.width + px) * 4) as usize;
                        if image.rgba8[offset..offset + 4] != background.rgba8[offset..offset + 4] {
                            visible[0] = visible[0].min(px);
                            visible[1] = visible[1].min(py);
                            visible[2] = visible[2].max(px + 1);
                            visible[3] = visible[3].max(py + 1);
                        }
                    }
                }
                // Test the final composited raster too, not only its layout math.
                assert!((visible[0] + visible[2]).abs_diff(SLOT * scale) <= 2);
                assert!((visible[1] + visible[3]).abs_diff(SLOT * scale) <= 2);
                // The number is still legible and the selected border is intact.
                assert!(image.rgba8.as_chunks::<4>().0.contains(&INK));
                assert_eq!(
                    &image.rgba8[..4],
                    if selection == Some(EquipmentSlot::One) {
                        &SELECTED
                    } else {
                        &BORDER
                    }
                );
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
