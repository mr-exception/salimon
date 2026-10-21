//! Bounded CPU bitmap presentation over the existing generic overlay contract.
use super::{Frame, Icon, Tone};
const COMPACT_WIDTH: u32 = 164;
const DETAIL_WIDTH: u32 = 284;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Layout {
    scale: u32,
    base_height: u32,
    width: u32,
    height: u32,
    details: bool,
    pub(super) page: usize,
    per_page: usize,
    pages: usize,
}
impl Layout {
    pub(super) fn new(
        viewport: [u32; 2],
        dpi: f64,
        details: bool,
        page: usize,
        count: usize,
        base_height: u32,
    ) -> Option<Self> {
        // Leave the lower half to inventory/equipment, reticle and global toasts.
        let available = [
            viewport[0].saturating_sub(32),
            (viewport[1] / 2).saturating_sub(16).min(
                viewport[1]
                    .saturating_sub(salimon_renderer::toolbar_message_inset(viewport, dpi) + 96),
            ),
        ];
        // Details reuse the meter band, moving all labels into the panel so
        // 640x360 remains readable above bottom inventory and messages.
        let base_height = if details {
            // At the native minimum size, details replace meter rows rather
            // than overlapping the bottom inventory or transient action text.
            if available[1] < 139 { 0 } else { 48 }
        } else {
            base_height
        };
        let desired = if dpi.is_finite() {
            (2.0 * dpi.clamp(1., 1.5)).round() as u32
        } else {
            2
        };
        let width = if details { DETAIL_WIDTH } else { 256 };
        let minimum = base_height + if details { 91 } else { 0 };
        let scale = desired
            .min(available[0] / width)
            .min(available[1] / minimum);
        if scale == 0 {
            return None;
        }
        let per_page = if details {
            ((available[1] / scale - base_height - 64) / 27).clamp(1, 4) as usize
        } else {
            1
        };
        let pages = count.div_ceil(per_page).max(1);
        let page = page.min(pages - 1);
        let rows = if details {
            count.saturating_sub(page * per_page).min(per_page).max(1)
        } else {
            0
        };
        let height = base_height + if details { 64 + 27 * rows as u32 } else { 0 };
        Some(Self {
            scale,
            base_height,
            width: width * scale,
            height: height * scale,
            details,
            page,
            per_page,
            pages,
        })
    }
}
struct Canvas {
    width: u32,
    height: u32,
    scale: u32,
    pixels: Vec<u8>,
}
impl Canvas {
    fn rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: [u8; 4]) {
        for yy in y * self.scale..((y + h) * self.scale).min(self.height) {
            for xx in x * self.scale..((x + w) * self.scale).min(self.width) {
                let i = ((yy * self.width + xx) * 4) as usize;
                self.pixels[i..i + 4].copy_from_slice(&color);
            }
        }
    }
    fn text(&mut self, x: u32, y: u32, text: &str, color: [u8; 4]) {
        for (n, ch) in text.chars().enumerate() {
            for (row, bits) in crate::action_bar::glyph_rows(ch).into_iter().enumerate() {
                for col in 0..5 {
                    if bits & (1 << (4 - col)) != 0 {
                        self.rect(x + n as u32 * 6 + col, y + row as u32, 1, 1, color);
                    }
                }
            }
        }
    }
    fn icon(&mut self, x: u32, y: u32, icon: Icon, color: [u8; 4]) {
        let rows: [u16; 9] = match icon {
            Icon::Health => [
                0b000111000,
                0b000111000,
                0b000111000,
                0b111111111,
                0b111111111,
                0b111111111,
                0b000111000,
                0b000111000,
                0b000111000,
            ],
            Icon::Hunger => [
                0b101010010,
                0b101010010,
                0b111110010,
                0b001000010,
                0b001000010,
                0b001000010,
                0b001000010,
                0b001000010,
                0b001000010,
            ],
            Icon::Fatigue => [
                0b000111100,
                0b001111000,
                0b011110000,
                0b111111110,
                0b000011100,
                0b000111000,
                0b001110000,
                0b011100000,
                0b011000000,
            ],
            Icon::Recovery => [
                0b001111100,
                0b010000010,
                0b100010001,
                0b100010001,
                0b101111101,
                0b100010001,
                0b100010001,
                0b010000010,
                0b001111100,
            ],
            Icon::Damage => [
                0b000010000,
                0b000111000,
                0b001111100,
                0b011101110,
                0b111101111,
                0b111111111,
                0b111101111,
                0b111111111,
                0b111111111,
            ],
            Icon::Amplify => [
                0b000010000,
                0b000111000,
                0b001111100,
                0b011010110,
                0b110010011,
                0b000010000,
                0b000010000,
                0b000010000,
                0b000010000,
            ],
            Icon::Wound => [
                0b100000001,
                0b010000010,
                0b001000100,
                0b000101000,
                0b000010000,
                0b000101000,
                0b001000100,
                0b010000010,
                0b100000001,
            ],
        };
        for (row, bits) in rows.into_iter().enumerate() {
            for col in 0..9 {
                if bits & (1 << (8 - col)) != 0 {
                    self.rect(x + col, y + row as u32, 1, 1, color);
                }
            }
        }
    }
}
const TEXT: [u8; 4] = salimon_renderer::ui_style::IVORY;
const MUTED: [u8; 4] = salimon_renderer::ui_style::MUTED;
const BACK: [u8; 4] = [8, 18, 27, 225];
const ACCENT: [u8; 4] = salimon_renderer::ui_style::CYAN;
pub(super) fn draw(frame: &Frame, layout: Layout) -> (u32, u32, Vec<u8>) {
    let mut c = Canvas {
        width: layout.width,
        height: layout.height,
        scale: layout.scale,
        pixels: vec![0; (layout.width * layout.height * 4) as usize],
    };
    let base_height = layout.base_height;
    c.rect(0, 0, COMPACT_WIDTH, base_height, BACK);
    c.rect(0, 0, 2, base_height, ACCENT);
    if base_height > 0 {
        for (i, meter) in frame.meters.iter().enumerate() {
            let y = 6 + i as u32 * 14;
            let color = match meter.tone {
                Tone::Normal => ACCENT,
                Tone::Low => [240, 187, 74, 255],
                Tone::Critical if frame.pulse => [255, 118, 98, 255],
                Tone::Critical => [194, 63, 59, 255],
            };
            c.icon(8, y, meter.icon, color);
            c.rect(25, y + 2, 128, 5, [46, 65, 76, 255]);
            c.rect(25, y + 2, 128 * u32::from(meter.fill) / 100, 5, color);
        }
    }
    if !layout.details {
        let mut y = 48;
        if !frame.statuses.is_empty() {
            for (i, status) in frame.statuses.iter().take(9).enumerate() {
                c.icon(8 + i as u32 * 13, y, status.icon, MUTED);
            }
            if frame.statuses.len() > 9 {
                c.text(128, y + 1, "+", MUTED);
            }
            y += 14;
        }
        // Penalty text is separate from icon/bar meter rows.
        for label in frame
            .penalties
            .iter()
            .take(2)
            .filter(|s| !s.ends_with(" -0%"))
        {
            c.text(8, y, label, TEXT);
            y += 10;
        }
        c.text(8, y, "H STATUS", MUTED);
        y += 12;
        if let Some(warning) = frame.warning {
            c.text(8, y, warning, [255, 155, 117, 255]);
        }
    }
    // Feedback floats beside the compact block, never across a meter or reticle.
    // With both gains present, separate rows remain inside the reserved bounds.
    for (i, gain) in frame.gains.iter().enumerate() {
        c.text(172, 30 + i as u32 * 12 - frame.rise, gain, ACCENT);
    }
    if layout.details {
        let top = base_height;
        c.rect(
            0,
            top,
            DETAIL_WIDTH,
            layout.height / layout.scale - top,
            BACK,
        );
        c.rect(0, top, DETAIL_WIDTH, 1, ACCENT);
        c.text(
            8,
            top + 5,
            &format!(
                "{} [ ] {}/{}",
                frame.warning.unwrap_or("H CLOSE"),
                layout.page + 1,
                layout.pages
            ),
            TEXT,
        );
        for (i, label) in frame.penalties.iter().enumerate() {
            c.text(8, top + 15 + i as u32 * 10, label, MUTED);
        }
        if frame.statuses.is_empty() {
            c.text(8, top + 64, "NO ACTIVE EFFECTS", MUTED);
        }
        for (i, status) in frame
            .statuses
            .iter()
            .skip(layout.page * layout.per_page)
            .take(layout.per_page)
            .enumerate()
        {
            let y = top + 60 + i as u32 * 27;
            c.icon(8, y, status.icon, ACCENT);
            // Long data-defined descriptions wrap; never truncate an effect.
            for (line, text) in description_lines(&status.description).iter().enumerate() {
                c.text(24, y + line as u32 * 8, text, TEXT);
            }
            c.text(24, y + 17, &status.duration, MUTED);
        }
    }
    (c.width, c.height, c.pixels)
}
pub(super) fn description_lines(text: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_ascii_whitespace() {
        if !line.is_empty() && line.len() + 1 + word.len() > 42 {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn layout_fits_above_lower_hud_at_supported_dpi_and_aspects_and_pages_all_effects() {
        for viewport in [
            [640, 360],
            [1280, 720],
            [1920, 1080],
            [2560, 1440],
            [1080, 1920],
        ] {
            for dpi in [1., 1.5, 2., f64::NAN] {
                for details in [false, true] {
                    let l = Layout::new(viewport, dpi, details, usize::MAX, 256, 104);
                    let Some(l) = l else {
                        assert!(
                            dpi.is_finite()
                                && (f64::from(viewport[0]) / dpi < 640.
                                    || f64::from(viewport[1]) / dpi < 360.)
                        );
                        continue;
                    };
                    assert!(
                        l.height + salimon_renderer::toolbar_message_inset(viewport, dpi) + 96
                            <= viewport[1]
                    );
                    assert!(l.width <= viewport[0] - 32);
                    assert!(l.height <= viewport[1] / 2);
                    assert_eq!(l.page, l.pages - 1);
                }
            }
        }
        assert!(Layout::new([0, 0], 1., true, 0, 1, 112).is_none());
    }
}
