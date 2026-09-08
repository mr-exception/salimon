//! Runtime-owned contextual action-bar state and deterministic bitmap generation.

use salimon_renderer::ui_style::{BACKGROUND, CYAN, IVORY};
use std::time::Duration;

const TRANSIENT_DURATION: Duration = Duration::from_secs(3);
// Shared by world interaction prompts, flight guidance and transient feedback.
// Integer bitmap scaling keeps glyph edges sharp. Cap high-DPI growth so
// contextual labels stay compact rather than doubling their physical footprint.
const BASE_SCALE: f64 = 2.0;
const MIN_SCALE: u32 = 2;
const MAX_SCALE: u32 = 3;

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
    combine_contextual: bool,
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
            combine_contextual: false,
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

    /// Tool requirements and placement errors must remain readable together.
    pub(crate) fn set_combined_contextual(&mut self, combined: bool) {
        if self.combine_contextual != combined {
            self.combine_contextual = combined;
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
        let combined = if self.combine_contextual {
            self.contextual
                .as_ref()
                .zip(self.transient)
                .map(|(context, (message, _))| format!("{context}\n{message}"))
        } else {
            None
        };
        let text = combined.as_deref().or(self
            .transient
            .map(|(message, _)| message)
            .or(self.contextual.as_deref()));
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

pub(crate) fn rasterize_action_bar(text: &str, scale: u32) -> (u32, u32, Vec<u8>) {
    const GLYPH_WIDTH: u32 = 5;
    const GLYPH_HEIGHT: u32 = 7;
    const CHARACTER_ADVANCE: u32 = 6;
    const HORIZONTAL_PADDING: u32 = 5;
    const VERTICAL_PADDING: u32 = 3;
    const BORDER: [u8; 4] = CYAN;
    const TEXT: [u8; 4] = IVORY;

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

// Runtime prose keeps the established uppercase label face. Instrument metric
// readings use the same face's explicit case-preserving SI variants directly.
pub(crate) fn glyph_rows(character: char) -> [u8; 7] {
    salimon_renderer::ui_style::glyph_rows(character)
}

#[cfg(test)]
mod tests {
    #[test]
    fn prompt_face_preserves_scanner_si_unit_case() {
        assert_ne!(super::glyph_rows('m'), super::glyph_rows('M'));
        assert_ne!(super::glyph_rows('s'), super::glyph_rows('S'));
        assert_eq!(
            super::glyph_rows('k'),
            salimon_renderer::ui_style::glyph_rows('k')
        );
    }
    #[test]
    fn tool_requirements_remain_visible_during_placement_feedback() {
        let mut bar = super::ActionBar::default();
        bar.set_contextual(Some("Iron 1kg / 10s".into()));
        bar.set_combined_contextual(true);
        bar.show_transient("Placeholder placed");
        assert_eq!(
            bar.rendered_text.as_deref(),
            Some("Iron 1kg / 10s\nPlaceholder placed")
        );
        bar.advance(std::time::Duration::from_secs(4));
        assert_eq!(bar.rendered_text.as_deref(), Some("Iron 1kg / 10s"));
        bar.set_combined_contextual(false);
        bar.show_transient("Door blocked");
        assert_eq!(bar.rendered_text.as_deref(), Some("Door blocked"));
    }

    use super::ActionBar;
    use std::time::Duration;

    #[test]
    fn recipe_selection_marker_renders_and_controls_share_the_scout_face() {
        let (w, _, pixels) = super::rasterize_action_bar("> #1 BASIC-RATION", 2);
        let marker_has_ink = (6..20).any(|y| {
            (10..20).any(|x| {
                let i = ((y * w + x) * 4) as usize;
                pixels[i..i + 4] == salimon_renderer::ui_style::IVORY
            })
        });
        assert!(
            marker_has_ink,
            "selected job marker must not be a blank glyph"
        );
        for c in "[E] USE COCKPIT > #1 + 20%".chars() {
            assert_eq!(
                super::glyph_rows(c),
                salimon_renderer::ui_style::glyph_rows(c)
            );
        }
    }

    #[test]
    #[ignore = "writes CPU prompt evidence; set SALIMON_SCOUT_UI_DIR"]
    fn scout_prompt_raster_evidence() {
        let dir = std::path::PathBuf::from(std::env::var("SALIMON_SCOUT_UI_DIR").unwrap());
        std::fs::create_dir_all(&dir).unwrap();
        let text = "[E] Open/close exit door\n[F] Drop carried cargo";
        for (name, dpi) in [("1x", 1.0), ("1_5x", 1.5), ("2x", 2.0)] {
            let mut bar = ActionBar::default();
            bar.set_scale_factor(dpi);
            bar.set_contextual(Some(text.to_owned()));
            let img = bar.image(true).unwrap();
            std::fs::write(dir.join(format!("prompt-{name}.rgba")), img.rgba8).unwrap();
            std::fs::write(
                dir.join(format!("prompt-{name}.size")),
                format!("{} {}", img.width, img.height),
            )
            .unwrap();
        }
    }

    #[test]
    fn dynamic_multiline_context_updates_mass_and_renders_both_rows() {
        let mut bar = ActionBar::default();
        bar.set_contextual(Some(
            "Silicate rock - approx 10.5 kg left\nHold left mouse to mine".to_owned(),
        ));
        let first = bar.image(true).unwrap();
        let revision = first.revision;
        assert_eq!(first.height, 44);
        let text_pixels_on_row = |row: usize| {
            first.rgba8[row * first.width as usize * 4..(row + 1) * first.width as usize * 4]
                .as_chunks::<4>()
                .0
                .contains(&salimon_renderer::ui_style::IVORY)
        };
        assert!(text_pixels_on_row(6));
        assert!(text_pixels_on_row(24));
        bar.set_contextual(Some(
            "Silicate rock - approx 9.5 kg left\nHold left mouse to mine".to_owned(),
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

    #[test]
    fn compact_prompt_typography_is_shared_and_tracks_bounded_dpi() {
        for text in [
            "Press L to land",
            "Press E to use",
            "Door locked while in flight",
        ] {
            let mut bar = ActionBar::default();
            bar.set_contextual(Some(text.to_owned()));
            let mut revision = bar.image(true).unwrap().revision;
            // Integer raster densities for 1×, 1.5× and 2× UI scale.
            for (dpi, scale) in [(1.0, 2), (1.5, 3), (2.0, 3)] {
                bar.set_scale_factor(dpi);
                let image = bar.image(true).unwrap();
                assert_eq!(image.height, 13 * scale);
                assert_eq!(image.width, (text.len() as u32 * 6 - 1 + 10) * scale);
                assert_eq!(image.rgba8.len(), (image.width * image.height * 4) as usize);
                let text_pixels = image
                    .rgba8
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .filter(|pixel| **pixel == salimon_renderer::ui_style::IVORY)
                    .count();
                assert!(text_pixels > 0);
                assert_eq!(text_pixels % (scale * scale) as usize, 0);
                assert!(image.revision >= revision);
                revision = image.revision;
                bar.set_scale_factor(dpi);
                assert_eq!(bar.image(true).unwrap().revision, revision);
            }
            // Transient and contextual messages share the same typography.
            let before = bar.image(true).unwrap().rgba8.to_vec();
            bar.show_transient(text);
            assert_eq!(bar.image(true).unwrap().rgba8, before);
        }
    }

    #[test]
    fn raster_is_smaller_than_issue_238_without_clipping_or_filtering() {
        for text in [
            "Press L to land",
            "Press E to use",
            "Press L to take off",
            "Silicate rock - approx 10.5 kg left\nHold left mouse to mine",
        ] {
            for (dpi, old_scale, new_scale) in [(1.0, 3, 2), (1.5, 5, 3), (2.0, 6, 3)] {
                let mut bar = ActionBar::default();
                bar.set_scale_factor(dpi);
                bar.set_contextual(Some(text.to_owned()));
                let image = bar.image(true).unwrap();
                let (old_width, old_height, _) = super::rasterize_action_bar(text, old_scale);
                assert!(image.width * 3 <= old_width * 2);
                assert!(image.height * 3 <= old_height * 2);

                // Every source cell survives as a solid integer-sized block,
                // including both rows and padding: no interpolation or lost glyphs.
                let (width, height, pixels) = super::rasterize_action_bar(text, 1);
                for y in 0..height {
                    for x in 0..width {
                        let source = ((y * width + x) * 4) as usize;
                        for dy in 0..new_scale {
                            for dx in 0..new_scale {
                                let target =
                                    (((y * new_scale + dy) * image.width + x * new_scale + dx) * 4)
                                        as usize;
                                assert_eq!(
                                    &image.rgba8[target..target + 4],
                                    &pixels[source..source + 4],
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn capped_dpi_reuses_raster_and_returning_to_one_x_rebuilds_it() {
        let mut bar = ActionBar::default();
        bar.set_contextual(Some("Press E to use".to_owned()));
        let one_x = bar.image(true).unwrap().rgba8.to_vec();
        bar.set_scale_factor(1.5);
        let revision = bar.image(true).unwrap().revision;
        bar.set_scale_factor(2.0);
        assert_eq!(bar.image(true).unwrap().revision, revision);
        bar.set_scale_factor(1.0);
        assert!(bar.image(true).unwrap().revision > revision);
        assert_eq!(bar.image(true).unwrap().rgba8, one_x);
    }

    #[test]
    fn invalid_and_out_of_range_dpi_keep_prompts_readable_and_bounded() {
        for (dpi, scale) in [
            (f64::NAN, 2),
            (f64::INFINITY, 2),
            (-1.0, 2),
            (0.0, 2),
            (99.0, 3),
        ] {
            let mut bar = ActionBar::default();
            bar.set_contextual(Some("Press E to use".to_owned()));
            bar.set_scale_factor(dpi);
            assert_eq!(bar.image(true).unwrap().height, 13 * scale);
        }
    }
}
