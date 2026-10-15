//! Gameplay aim presentation; equipment authority stays in `EquipmentToolbar`.
use salimon_renderer::{OverlayImage, OverlayPlacement};

use crate::equipment::EquipmentToolbar;

const SIZE: u32 = 17;
const BYTE_COUNT: usize = (SIZE * SIZE * 4) as usize;
const DOT_SCALE: i32 = 3;
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
            // Replicate each original dot pixel as a centered 3×3 block.
            // This triples both the white diameter (3→9) and outline (5→15)
            // without scaling the equipped cross or filtering either image.
            let dot_dx = (dx + DOT_SCALE / 2) / DOT_SCALE;
            let dot_dy = (dy + DOT_SCALE / 2) / DOT_SCALE;
            let dot_distance_squared = dot_dx * dot_dx + dot_dy * dot_dy;
            let white = if cross {
                (dx == 0 && dy <= 6) || (dy == 0 && dx <= 6)
            } else {
                dot_distance_squared <= 1
            };
            let outline = if cross {
                (dx <= 1 && dy <= 7) || (dy <= 1 && dx <= 7)
            } else {
                dot_distance_squared <= 5
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
        assert_eq!(&DOT[arm..arm + 4], &[24, 24, 24, 255]);
        assert_eq!(CROSS[arm + 3], 255);
    }

    #[test]
    fn dot_triples_diameters_and_area_while_cross_keeps_its_original_size() {
        let bounds_and_count = |rgba: &[u8], white: bool| {
            let mut min = [SIZE; 2];
            let mut max = [0; 2];
            let mut count = 0;
            for y in 0..SIZE {
                for x in 0..SIZE {
                    let offset = ((y * SIZE + x) * 4) as usize;
                    if rgba[offset + 3] != 0 && (!white || rgba[offset] == 240) {
                        min = [min[0].min(x), min[1].min(y)];
                        max = [max[0].max(x), max[1].max(y)];
                        count += 1;
                    }
                }
            }
            (min, max, count)
        };
        // Original dot: white diameter 3 / five pixels, outlined diameter 5 /
        // 21 pixels. The enlarged mask has 3× diameter and 9× pixel area.
        assert_eq!(bounds_and_count(&DOT, true), ([4, 4], [12, 12], 45));
        assert_eq!(bounds_and_count(&DOT, false), ([1, 1], [15, 15], 189));
        assert_eq!(bounds_and_count(&CROSS, true), ([2, 2], [14, 14], 25));
        assert_eq!(bounds_and_count(&CROSS, false), ([1, 1], [15, 15], 81));
    }
}
