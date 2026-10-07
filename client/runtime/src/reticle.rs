//! Gameplay aim presentation; equipment authority stays in `EquipmentToolbar`.
use salimon_renderer::{OverlayImage, OverlayPlacement};

use crate::equipment::EquipmentToolbar;

const SIZE: u32 = 17;
const BYTE_COUNT: usize = (SIZE * SIZE * 4) as usize;
const DOT: [u8; BYTE_COUNT] = pixels(false);
const CROSS: [u8; BYTE_COUNT] = pixels(true);

// Symmetric white strokes with a dark one-pixel outline stay readable on both
// bright surfaces and space. The texture midpoint is the camera projection ray.
const fn pixels(cross: bool) -> [u8; BYTE_COUNT] {
    let mut rgba = [0; BYTE_COUNT];
    let mut y = 0;
    while y < SIZE {
        let mut x = 0;
        while x < SIZE {
            let dx = (x as i32 - (SIZE / 2) as i32).abs();
            let dy = (y as i32 - (SIZE / 2) as i32).abs();
            let white = if cross {
                (dx == 0 && dy <= 6) || (dy == 0 && dx <= 6)
            } else {
                dx * dx + dy * dy <= 1
            };
            let outline = if cross {
                (dx <= 1 && dy <= 7) || (dy <= 1 && dx <= 7)
            } else {
                dx * dx + dy * dy <= 5
            };
            let offset = ((y * SIZE + x) * 4) as usize;
            if white || outline {
                let shade = if white { 240 } else { 24 };
                rgba[offset] = shade;
                rgba[offset + 1] = shade;
                rgba[offset + 2] = shade;
                rgba[offset + 3] = 255;
            }
            x += 1;
        }
        y += 1;
    }
    rgba
}

pub(crate) fn image(gameplay: bool, tool: &EquipmentToolbar) -> Option<OverlayImage<'static>> {
    gameplay.then_some(OverlayImage {
        width: SIZE,
        height: SIZE,
        rgba8: if tool.mining_equipped() { &CROSS } else { &DOT },
        revision: u64::from(tool.mining_equipped()),
        placement: OverlayPlacement::Center,
    })
}

#[cfg(test)]
mod tests {
    use super::{CROSS, DOT, SIZE, image};
    use crate::equipment::EquipmentToolbar;

    #[test]
    fn equip_and_stow_select_the_current_authoritative_state() {
        let mut tool = EquipmentToolbar::default();
        let empty = image(true, &tool).unwrap();
        assert_eq!(empty.rgba8, DOT);
        tool.select(crate::equipment::ToolbarSlot::One, false);
        let equipped = image(true, &tool).unwrap();
        assert_eq!(equipped.rgba8, CROSS);
        assert_ne!(equipped.revision, empty.revision);
        tool.sync_carrying(true);
        let stowed = image(true, &tool).unwrap();
        assert_eq!(stowed.rgba8, empty.rgba8);
        assert_eq!(stowed.revision, empty.revision);
        assert!(image(false, &tool).is_none());
    }

    #[test]
    fn both_shapes_are_symmetric_about_the_same_center() {
        for rgba in [&DOT, &CROSS] {
            let pixel = |x, y| {
                let offset = ((y * SIZE + x) * 4) as usize;
                &rgba[offset..offset + 4]
            };
            for y in 0..SIZE {
                for x in 0..SIZE {
                    assert_eq!(pixel(x, y), pixel(SIZE - 1 - x, y));
                    assert_eq!(pixel(x, y), pixel(x, SIZE - 1 - y));
                }
            }
            assert_eq!(pixel(SIZE / 2, SIZE / 2), [240, 240, 240, 255]);
        }
        let arm = ((SIZE / 2 * SIZE + SIZE / 2 + 6) * 4) as usize;
        assert_eq!(DOT[arm + 3], 0);
        assert_eq!(CROSS[arm + 3], 255);
    }
}
