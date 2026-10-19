//! Renderer-neutral radial presentation and matching physical-pixel hit testing.
use crate::{ConsumableIcon, OverlayImage, OverlayPlacement};
use std::f64::consts::{FRAC_PI_2, TAU};
const SIZE: u32 = 480;
const INNER: f64 = 80.0;
const OUTER: f64 = 180.0;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConsumableWheel {
    pub slots: [Option<ConsumableIcon>; 5],
    pub hovered: Option<usize>,
    pub names: [String; 5],
    pub effect: String,
}
#[derive(Default)]
pub struct ConsumableWheelRaster {
    key: Option<ConsumableWheel>,
    pixels: Vec<u8>,
    revision: u64,
}
fn sector(point: [f64; 2]) -> Option<usize> {
    let dx = point[0] - 240.0;
    let dy = point[1] - 240.0;
    let radius = dx.hypot(dy);
    if !radius.is_finite() || !(INNER..OUTER).contains(&radius) {
        return None;
    }
    let angle = (dy.atan2(dx) + FRAC_PI_2 + TAU / 10.0).rem_euclid(TAU);
    // Leave a small visible and noninteractive separator at each boundary.
    let position = angle / (TAU / 5.0);
    if !(0.015..=0.985).contains(&position.fract()) {
        return None;
    }
    Some(position.floor() as usize)
}
/// Same uniform fit and center as OverlayPlacement::Center, including tiny windows.
pub fn consumable_wheel_hit_test(surface: [u32; 2], pointer: [f64; 2]) -> Option<usize> {
    let size = crate::overlay::fitted_overlay_size(surface, [SIZE, SIZE]);
    if size[0] <= 0.0 || size[1] <= 0.0 {
        return None;
    }
    let point = std::array::from_fn(|i| {
        (pointer[i] - (f64::from(surface[i]) - f64::from(size[i])) / 2.0) * f64::from(SIZE)
            / f64::from(size[i])
    });
    sector(point)
}
impl ConsumableWheelRaster {
    pub fn update(&mut self, state: ConsumableWheel) {
        if self.key.as_ref() == Some(&state) {
            return;
        }
        self.pixels = vec![0; (SIZE * SIZE * 4) as usize];
        self.revision = self.revision.wrapping_add(1);
        for y in 0..SIZE {
            for x in 0..SIZE {
                if let Some(index) = sector([f64::from(x) + 0.5, f64::from(y) + 0.5]) {
                    let color = if state.slots[index].is_none() {
                        [21, 28, 34, 224]
                    } else if state.hovered == Some(index) {
                        [32, 102, 110, 250]
                    } else {
                        [13, 37, 48, 240]
                    };
                    self.pixel(x, y, color);
                }
            }
        }
        for index in 0..5 {
            let angle = index as f64 * TAU / 5.0 - FRAC_PI_2;
            let x = (240.0 + 130.0 * angle.cos()) as u32;
            let y = (240.0 + 130.0 * angle.sin()) as u32;
            if let Some(icon) = state.slots[index] {
                self.rect(
                    x - 13,
                    y - 13,
                    26,
                    26,
                    match icon {
                        ConsumableIcon::Food => [197, 139, 57, 255],
                        ConsumableIcon::Medical => [214, 229, 222, 255],
                    },
                );
                self.rect(x - 13, y - 13, 26, 4, [245, 225, 173, 255]);
                match icon {
                    ConsumableIcon::Food => {
                        self.rect(x - 10, y - 4, 20, 4, [104, 68, 34, 255]);
                        self.rect(x - 10, y + 5, 20, 3, [104, 68, 34, 255]);
                    }
                    ConsumableIcon::Medical => {
                        self.rect(x - 3, y - 8, 6, 19, [184, 43, 45, 255]);
                        self.rect(x - 9, y - 2, 18, 6, [184, 43, 45, 255]);
                    }
                }
            } else {
                self.text("EMPTY", x - 15, y - 3);
            }
        }
        self.text("HOLD Q", 222, 224);
        self.text("CLICK USE", 213, 240);
        self.text("TAB / ENTER", 207, 256);
        let label = state
            .hovered
            .and_then(|i| state.names.get(i))
            .map_or("SELECT A SLOT", String::as_str);
        self.center_text(label, 432);
        self.center_text(&state.effect, 450);
        self.key = Some(state);
    }
    fn pixel(&mut self, x: u32, y: u32, color: [u8; 4]) {
        if x < SIZE && y < SIZE {
            let offset = ((y * SIZE + x) * 4) as usize;
            self.pixels[offset..offset + 4].copy_from_slice(&color);
        }
    }
    fn rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: [u8; 4]) {
        for py in y..y + h {
            for px in x..x + w {
                self.pixel(px, py, color);
            }
        }
    }
    fn text(&mut self, text: &str, x: u32, y: u32) {
        for (index, ch) in text.chars().take(72).enumerate() {
            for (row, bits) in crate::cockpit_instruments::glyph_rows(ch.to_ascii_uppercase())
                .iter()
                .enumerate()
            {
                for col in 0..5 {
                    if bits & (1 << (4 - col)) != 0 {
                        self.pixel(
                            x + index as u32 * 6 + col,
                            y + row as u32,
                            [248, 250, 252, 255],
                        );
                    }
                }
            }
        }
    }
    fn center_text(&mut self, text: &str, y: u32) {
        let width = text.chars().count().min(72) as u32 * 6;
        self.text(text, (SIZE - width) / 2, y);
    }
    pub fn image(&self) -> Option<OverlayImage<'_>> {
        self.key.as_ref()?;
        Some(OverlayImage {
            width: SIZE,
            height: SIZE,
            rgba8: &self.pixels,
            revision: self.revision,
            placement: OverlayPlacement::Center,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn centers_fit_and_hit_every_segment_at_all_drawable_sizes() {
        for surface in [[1280, 720], [1920, 1080], [320, 240], [640, 360]] {
            let fitted = crate::overlay::fitted_overlay_size(surface, [SIZE, SIZE]);
            for index in 0..5 {
                let angle = index as f64 * TAU / 5.0 - FRAC_PI_2;
                let point = [
                    surface[0] as f64 / 2.0 + 130.0 * angle.cos() * fitted[0] as f64 / SIZE as f64,
                    surface[1] as f64 / 2.0 + 130.0 * angle.sin() * fitted[1] as f64 / SIZE as f64,
                ];
                assert_eq!(consumable_wheel_hit_test(surface, point), Some(index));
            }
            assert_eq!(
                consumable_wheel_hit_test(
                    surface,
                    [surface[0] as f64 / 2.0, surface[1] as f64 / 2.0]
                ),
                None
            );
            assert_eq!(consumable_wheel_hit_test(surface, [0.0, 0.0]), None);
        }
        assert_eq!(consumable_wheel_hit_test([0, 0], [0.0, 0.0]), None);
        assert_eq!(consumable_wheel_hit_test([480, 480], [f64::NAN, 1.0]), None);
    }
    #[test]
    fn empty_slots_icons_hover_names_effects_and_cache_are_present() {
        let mut raster = ConsumableWheelRaster::default();
        let mut state = ConsumableWheel {
            slots: [
                Some(ConsumableIcon::Food),
                None,
                None,
                Some(ConsumableIcon::Medical),
                None,
            ],
            hovered: Some(0),
            names: std::array::from_fn(|i| format!("ITEM {i}")),
            effect: "NO ACTIVE EFFECT".into(),
        };
        raster.update(state.clone());
        raster.image().unwrap().validate().unwrap();
        let old = raster.pixels.clone();
        let revision = raster.revision;
        raster.update(state.clone());
        assert_eq!(revision, raster.revision);
        state.hovered = Some(3);
        raster.update(state);
        assert_ne!(revision, raster.revision);
        assert_ne!(old, raster.pixels);
    }
}
