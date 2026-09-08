use super::{BUTTON, Layout, PANEL, Report};
const INK: [u8; 4] = [230, 241, 245, 255];
const MUTED: [u8; 4] = [148, 176, 188, 255];
const ACCENT: [u8; 4] = [64, 215, 191, 255];
struct Canvas {
    width: u32,
    height: u32,
    origin: [u32; 2],
    scale: u32,
    pixels: Vec<u8>,
}
impl Canvas {
    fn rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: [u8; 4]) {
        for yy in self.origin[1] + y * self.scale
            ..(self.origin[1] + (y + h) * self.scale).min(self.height)
        {
            for xx in self.origin[0] + x * self.scale
                ..(self.origin[0] + (x + w) * self.scale).min(self.width)
            {
                let offset = ((yy * self.width + xx) * 4) as usize;
                self.pixels[offset..offset + 4].copy_from_slice(&color);
            }
        }
    }
    fn text(&mut self, x: u32, y: u32, text: &str, color: [u8; 4], size: u32) {
        for (n, ch) in text.chars().enumerate() {
            for (row, bits) in crate::action_bar::glyph_rows(ch).into_iter().enumerate() {
                for col in 0..5 {
                    if bits & (1 << (4 - col)) != 0 {
                        self.rect(
                            x + (n as u32 * 6 + col) * size,
                            y + row as u32 * size,
                            size,
                            size,
                            color,
                        );
                    }
                }
            }
        }
    }
    fn centered(&mut self, y: u32, text: &str, color: [u8; 4], size: u32) {
        let width = (text.len() as u32 * 6 - 1) * size;
        self.text((PANEL[0] - width) / 2, y, text, color, size);
    }
}
pub(super) fn draw(report: &Report, layout: Layout, hover: bool, failed: bool) -> Vec<u8> {
    let mut c = Canvas {
        width: layout.canvas[0],
        height: layout.canvas[1],
        origin: layout.panel_origin,
        scale: layout.scale,
        pixels: vec![0; (layout.canvas[0] * layout.canvas[1] * 4) as usize],
    };
    for p in c.pixels.as_chunks_mut::<4>().0 {
        p.copy_from_slice(&[3, 9, 15, 205]);
    }
    c.rect(0, 0, PANEL[0], PANEL[1], [9, 21, 31, 255]);
    c.rect(0, 0, PANEL[0], 1, ACCENT);
    c.rect(0, PANEL[1] - 1, PANEL[0], 1, [39, 85, 91, 255]);
    for x in [0, PANEL[0] - 1] {
        c.rect(x, 0, 1, 12, ACCENT);
        c.rect(x, PANEL[1] - 12, 1, 12, ACCENT);
    }
    c.centered(6, "SALIMON / FINAL TRANSMISSION", MUTED, 1);
    c.centered(19, "JOURNEY ENDED", INK, 2);
    c.rect(16, 39, 248, 1, [37, 64, 76, 255]);
    for (i, (label, value)) in [
        ("CAUSE", &report.cause),
        ("ALIVE", &report.duration),
        ("TRAVEL", &report.distance),
        ("LOCATION", &report.location),
    ]
    .into_iter()
    .enumerate()
    {
        c.text(16, 48 + i as u32 * 16, label, MUTED, 1);
        c.text(86, 48 + i as u32 * 16, value, INK, 1);
    }
    let [x, y, w, h] = BUTTON;
    c.rect(x, y, w, h, if hover { ACCENT } else { [28, 66, 71, 255] });
    c.rect(x, y, w, 1, ACCENT);
    c.rect(x, y + h - 1, w, 1, ACCENT);
    c.centered(
        y + 7,
        "RESTART GAME",
        if hover { [5, 25, 30, 255] } else { INK },
        1,
    );
    c.centered(141, "ENTER / R OR CLICK", MUTED, 1);
    if failed {
        c.centered(
            152,
            "RESTART FAILED - CHECK SAVE STORAGE",
            [255, 160, 123, 255],
            1,
        );
    }
    c.pixels
}
