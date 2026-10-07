//! Runtime-owned contextual action-bar state and deterministic bitmap generation.

use std::time::Duration;

const TRANSIENT_DURATION: Duration = Duration::from_secs(3);
const BASE_SCALE: f64 = 4.0;
const MIN_SCALE: u32 = 4;
const MAX_SCALE: u32 = 8;

#[derive(Clone, Copy, Debug)]
pub(crate) struct ActionBarImage<'a> {
    pub width: u32,
    pub height: u32,
    pub rgba8: &'a [u8],
    pub revision: u64,
}

#[derive(Debug)]
pub(crate) struct ActionBar {
    contextual: Option<String>,
    transient: Option<(&'static str, Duration)>,
    rendered_text: Option<String>,
    scale: u32,
    width: u32,
    height: u32,
    pixels: Vec<u8>,
    revision: u64,
}

impl Default for ActionBar {
    fn default() -> Self {
        Self {
            contextual: None,
            transient: None,
            rendered_text: None,
            scale: MIN_SCALE,
            width: 0,
            height: 0,
            pixels: Vec::new(),
            revision: 0,
        }
    }
}

impl ActionBar {
    pub(crate) fn set_scale_factor(&mut self, scale_factor: f64) {
        let finite_scale = if scale_factor.is_finite() {
            scale_factor
        } else {
            1.0
        };
        let scale = (finite_scale.clamp(1.0, 2.0) * BASE_SCALE).round() as u32;
        let scale = scale.clamp(MIN_SCALE, MAX_SCALE);
        if self.scale != scale {
            self.scale = scale;
            self.rebuild_if_needed(true);
        }
    }

    pub(crate) fn set_contextual(&mut self, message: Option<String>) {
        if self.contextual != message {
            self.contextual = message;
            self.rebuild_if_needed(false);
        }
    }

    pub(crate) fn show_transient(&mut self, message: &'static str) {
        self.transient = Some((message, TRANSIENT_DURATION));
        self.rebuild_if_needed(false);
    }

    pub(crate) fn advance(&mut self, delta: Duration) {
        let Some((message, remaining)) = self.transient else {
            return;
        };
        let remaining = remaining.saturating_sub(delta);
        self.transient = (!remaining.is_zero()).then_some((message, remaining));
        self.rebuild_if_needed(false);
    }

    pub(crate) fn has_transient(&self) -> bool {
        self.transient.is_some()
    }

    pub(crate) fn image(&self, visible: bool) -> Option<ActionBarImage<'_>> {
        (visible && self.rendered_text.is_some()).then_some(ActionBarImage {
            width: self.width,
            height: self.height,
            rgba8: &self.pixels,
            revision: self.revision,
        })
    }

    fn rebuild_if_needed(&mut self, force: bool) {
        let text = self
            .transient
            .map(|(message, _)| message)
            .or(self.contextual.as_deref());
        if !force && self.rendered_text.as_deref() == text {
            return;
        }
        self.rendered_text = text.map(str::to_owned);
        if let Some(text) = text {
            (self.width, self.height, self.pixels) = rasterize_action_bar(text, self.scale);
        } else {
            self.width = 0;
            self.height = 0;
            self.pixels.clear();
        }
        self.revision = self.revision.saturating_add(1);
    }
}

fn rasterize_action_bar(text: &str, scale: u32) -> (u32, u32, Vec<u8>) {
    const GLYPH_WIDTH: u32 = 5;
    const GLYPH_HEIGHT: u32 = 7;
    const CHARACTER_ADVANCE: u32 = 6;
    const HORIZONTAL_PADDING: u32 = 5;
    const VERTICAL_PADDING: u32 = 3;
    const BACKGROUND: [u8; 4] = [7, 16, 29, 235];
    const BORDER: [u8; 4] = [45, 212, 191, 255];
    const TEXT: [u8; 4] = [248, 250, 252, 255];

    let lines: Vec<_> = text.lines().collect();
    let characters = lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(1)
        .max(1) as u32;
    let content_width = characters
        .saturating_mul(CHARACTER_ADVANCE)
        .saturating_sub(CHARACTER_ADVANCE - GLYPH_WIDTH);
    let width = (content_width + HORIZONTAL_PADDING * 2) * scale;
    let height =
        (lines.len().max(1) as u32 * (GLYPH_HEIGHT + 2) - 2 + VERTICAL_PADDING * 2) * scale;
    let mut pixels = vec![0_u8; width as usize * height as usize * 4];
    for pixel in pixels.as_chunks_mut::<4>().0 {
        pixel.copy_from_slice(&BACKGROUND);
    }
    fill_rect(&mut pixels, width, 0, 0, width, scale, BORDER);
    fill_rect(&mut pixels, width, 0, height - scale, width, scale, BORDER);

    for (line_index, line) in lines.iter().enumerate() {
        for (index, character) in line.chars().enumerate() {
            let glyph_x = (HORIZONTAL_PADDING + index as u32 * CHARACTER_ADVANCE) * scale;
            let glyph_y = (VERTICAL_PADDING + line_index as u32 * (GLYPH_HEIGHT + 2)) * scale;
            for (row, bits) in glyph_rows(character).into_iter().enumerate() {
                for column in 0..GLYPH_WIDTH {
                    if bits & (1 << (GLYPH_WIDTH - 1 - column)) != 0 {
                        fill_rect(
                            &mut pixels,
                            width,
                            glyph_x + column * scale,
                            glyph_y + row as u32 * scale,
                            scale,
                            scale,
                            TEXT,
                        );
                    }
                }
            }
        }
    }
    (width, height, pixels)
}

#[allow(clippy::too_many_arguments)]
fn fill_rect(
    pixels: &mut [u8],
    image_width: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color: [u8; 4],
) {
    let image_height = pixels.len() as u32 / 4 / image_width;
    for pixel_y in y..y.saturating_add(height).min(image_height) {
        for pixel_x in x..x.saturating_add(width).min(image_width) {
            let offset = (pixel_y as usize * image_width as usize + pixel_x as usize) * 4;
            pixels[offset..offset + 4].copy_from_slice(&color);
        }
    }
}

fn glyph_rows(character: char) -> [u8; 7] {
    match character.to_ascii_uppercase() {
        'A' => [14, 17, 17, 31, 17, 17, 17],
        'B' => [30, 17, 17, 30, 17, 17, 30],
        'C' => [15, 16, 16, 16, 16, 16, 15],
        'D' => [30, 17, 17, 17, 17, 17, 30],
        'E' => [31, 16, 16, 30, 16, 16, 31],
        'F' => [31, 16, 16, 30, 16, 16, 16],
        'G' => [15, 16, 16, 23, 17, 17, 15],
        'H' => [17, 17, 17, 31, 17, 17, 17],
        'I' => [31, 4, 4, 4, 4, 4, 31],
        'J' => [7, 2, 2, 2, 18, 18, 12],
        'K' => [17, 18, 20, 24, 20, 18, 17],
        'L' => [16, 16, 16, 16, 16, 16, 31],
        'M' => [17, 27, 21, 21, 17, 17, 17],
        'N' => [17, 25, 21, 19, 17, 17, 17],
        'O' => [14, 17, 17, 17, 17, 17, 14],
        'P' => [30, 17, 17, 30, 16, 16, 16],
        'Q' => [14, 17, 17, 17, 21, 18, 13],
        'R' => [30, 17, 17, 30, 20, 18, 17],
        'S' => [15, 16, 16, 14, 1, 1, 30],
        'T' => [31, 4, 4, 4, 4, 4, 4],
        'U' => [17, 17, 17, 17, 17, 17, 14],
        'V' => [17, 17, 17, 17, 17, 10, 4],
        'W' => [17, 17, 17, 21, 21, 21, 10],
        'X' => [17, 17, 10, 4, 10, 17, 17],
        'Y' => [17, 17, 10, 4, 4, 4, 4],
        'Z' => [31, 1, 2, 4, 8, 16, 31],
        '0' => [14, 17, 19, 21, 25, 17, 14],
        '1' => [4, 12, 4, 4, 4, 4, 14],
        '2' => [14, 17, 1, 2, 4, 8, 31],
        '3' => [30, 1, 1, 14, 1, 1, 30],
        '4' => [2, 6, 10, 18, 31, 2, 2],
        '5' => [31, 16, 16, 30, 1, 1, 30],
        '6' => [14, 16, 16, 30, 17, 17, 14],
        '7' => [31, 1, 2, 4, 8, 8, 8],
        '8' => [14, 17, 17, 14, 17, 17, 14],
        '9' => [14, 17, 17, 15, 1, 1, 14],
        '.' => [0, 0, 0, 0, 0, 6, 6],
        '/' => [1, 2, 2, 4, 8, 8, 16],
        '%' => [17, 2, 4, 8, 17, 0, 0],
        '-' => [0, 0, 0, 31, 0, 0, 0],
        _ => [0; 7],
    }
}

#[cfg(test)]
mod tests {
    use super::ActionBar;
    use std::time::Duration;

    #[test]
    fn dynamic_multiline_context_updates_mass_and_renders_both_rows() {
        let mut bar = ActionBar::default();
        bar.set_contextual(Some(
            "Silicate rock - approx 10.5 kg left\nHold F to mine".to_owned(),
        ));
        let first = bar.image(true).unwrap();
        let revision = first.revision;
        assert_eq!(first.height, 88);
        let text_pixels_on_row = |row: usize| {
            first.rgba8[row * first.width as usize * 4..(row + 1) * first.width as usize * 4]
                .as_chunks::<4>()
                .0
                .contains(&[248, 250, 252, 255])
        };
        assert!(text_pixels_on_row(12));
        assert!(text_pixels_on_row(48));
        bar.set_contextual(Some(
            "Silicate rock - approx 9.5 kg left\nHold F to mine".to_owned(),
        ));
        assert!(bar.image(true).unwrap().revision > revision);
        bar.show_transient("Only one world object - G to place/drop first");
        bar.advance(Duration::from_secs(3));
        assert!(bar.rendered_text.as_deref().unwrap().contains("9.5 kg"));
        bar.set_contextual(None);
        assert!(bar.image(true).is_none());
    }

    #[test]
    fn contextual_prompts_appear_change_and_disappear() {
        let mut bar = ActionBar::default();
        assert!(bar.image(true).is_none());

        bar.set_contextual(Some("Press L to land".to_owned()));
        let landing_revision = bar.image(true).expect("landing action is visible").revision;
        bar.set_contextual(Some("Press L to take off".to_owned()));
        assert!(bar.image(true).expect("takeoff action is visible").revision > landing_revision);

        bar.set_contextual(None);
        assert!(bar.image(true).is_none());
    }

    #[test]
    fn blocked_action_temporarily_overrides_then_restores_context() {
        let mut bar = ActionBar::default();
        bar.set_contextual(Some("Press L to take off".to_owned()));
        bar.show_transient("Door locked while in flight");
        assert_eq!(
            bar.rendered_text.as_deref(),
            Some("Door locked while in flight")
        );

        bar.advance(Duration::from_secs(2));
        assert_eq!(
            bar.rendered_text.as_deref(),
            Some("Door locked while in flight")
        );
        bar.advance(Duration::from_secs(1));
        assert_eq!(bar.rendered_text.as_deref(), Some("Press L to take off"));
    }

    #[test]
    fn action_bar_is_hidden_outside_normal_gameplay_view() {
        let mut bar = ActionBar::default();
        bar.set_contextual(Some("Close door before takeoff".to_owned()));
        assert!(bar.image(true).is_some());
        assert!(bar.image(false).is_none());
    }
}
