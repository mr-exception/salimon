//! Embedded bitmap font and CPU RGBA panel rasterization.

pub(super) fn rasterize_panel(text: &str, scale: u32) -> (u32, u32, Vec<u8>) {
    const GLYPH_WIDTH: u32 = 5;
    const GLYPH_HEIGHT: u32 = 7;
    const CHARACTER_ADVANCE: u32 = 6;
    const LINE_ADVANCE: u32 = 9;
    const PADDING: u32 = 3;
    const BACKGROUND: [u8; 4] = [5, 12, 24, 226];
    const BORDER: [u8; 4] = [34, 211, 238, 255];
    const HEADING: [u8; 4] = [103, 232, 249, 255];
    const MUTED: [u8; 4] = [148, 163, 184, 255];
    const TEXT: [u8; 4] = [226, 232, 240, 255];

    let lines: Vec<&str> = text.lines().collect();
    let max_characters = lines
        .iter()
        .map(|line| line.chars().count() as u32)
        .max()
        .unwrap_or(1);
    let line_count = lines.len().max(1) as u32;
    let content_width = max_characters
        .saturating_mul(CHARACTER_ADVANCE)
        .saturating_sub(CHARACTER_ADVANCE - GLYPH_WIDTH);
    let content_height = line_count
        .saturating_mul(LINE_ADVANCE)
        .saturating_sub(LINE_ADVANCE - GLYPH_HEIGHT);
    let width = (content_width + PADDING * 2) * scale;
    let height = (content_height + PADDING * 2) * scale;
    let pixel_count = width as usize * height as usize;
    let mut pixels = vec![0_u8; pixel_count * 4];
    for pixel in pixels.as_chunks_mut::<4>().0 {
        pixel.copy_from_slice(&BACKGROUND);
    }

    fill_rect(&mut pixels, width, height, 0, 0, width, scale, BORDER);
    fill_rect(&mut pixels, width, height, 0, 0, scale, height, BORDER);
    fill_rect(
        &mut pixels,
        width,
        height,
        0,
        height - scale,
        width,
        scale,
        BORDER,
    );
    fill_rect(
        &mut pixels,
        width,
        height,
        width - scale,
        0,
        scale,
        height,
        BORDER,
    );

    for (line_index, line) in lines.iter().enumerate() {
        let color = match line_index {
            0 => HEADING,
            1 => MUTED,
            _ => TEXT,
        };
        let glyph_y = (PADDING + line_index as u32 * LINE_ADVANCE) * scale;
        for (character_index, character) in line.chars().enumerate() {
            if character == ' ' {
                continue;
            }
            let glyph_x = (PADDING + character_index as u32 * CHARACTER_ADVANCE) * scale;
            for (row, bits) in glyph_rows(character).into_iter().enumerate() {
                for column in 0..GLYPH_WIDTH {
                    if bits & (1 << (GLYPH_WIDTH - 1 - column)) != 0 {
                        fill_rect(
                            &mut pixels,
                            width,
                            height,
                            glyph_x + column * scale,
                            glyph_y + row as u32 * scale,
                            scale,
                            scale,
                            color,
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
    image_height: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color: [u8; 4],
) {
    let end_x = x.saturating_add(width).min(image_width);
    let end_y = y.saturating_add(height).min(image_height);
    for pixel_y in y.min(image_height)..end_y {
        for pixel_x in x.min(image_width)..end_x {
            let offset = (pixel_y as usize * image_width as usize + pixel_x as usize) * 4;
            pixels[offset..offset + 4].copy_from_slice(&color);
        }
    }
}

fn glyph_rows(character: char) -> [u8; 7] {
    match character.to_ascii_uppercase() {
        'A' => [
            0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'B' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
        ],
        'C' => [
            0b01111, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b01111,
        ],
        'D' => [
            0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
        ],
        'E' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ],
        'F' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'G' => [
            0b01111, 0b10000, 0b10000, 0b10111, 0b10001, 0b10001, 0b01111,
        ],
        'H' => [
            0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'I' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b11111,
        ],
        'J' => [
            0b00111, 0b00010, 0b00010, 0b00010, 0b10010, 0b10010, 0b01100,
        ],
        'K' => [
            0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
        ],
        'L' => [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
        ],
        'M' => [
            0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
        ],
        'N' => [
            0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001,
        ],
        'O' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'P' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'Q' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101,
        ],
        'R' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
        ],
        'S' => [
            0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        'T' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'U' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'V' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100,
        ],
        'W' => [
            0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b10101, 0b01010,
        ],
        'X' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001,
        ],
        'Y' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'Z' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111,
        ],
        '0' => [
            0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
        ],
        '1' => [
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        '2' => [
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
        ],
        '3' => [
            0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        '4' => [
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ],
        '5' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b00001, 0b00001, 0b11110,
        ],
        '6' => [
            0b01110, 0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
        ],
        '7' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
        ],
        '8' => [
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ],
        '9' => [
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00001, 0b01110,
        ],
        ':' => [0, 0b00100, 0b00100, 0, 0b00100, 0b00100, 0],
        '.' => [0, 0, 0, 0, 0, 0b00110, 0b00110],
        ',' => [0, 0, 0, 0, 0b00110, 0b00100, 0b01000],
        '-' => [0, 0, 0, 0b11111, 0, 0, 0],
        '/' => [
            0b00001, 0b00010, 0b00010, 0b00100, 0b01000, 0b01000, 0b10000,
        ],
        '%' => [0b11001, 0b11010, 0b00100, 0b01000, 0b10110, 0b00110, 0],
        '(' => [
            0b00010, 0b00100, 0b01000, 0b01000, 0b01000, 0b00100, 0b00010,
        ],
        ')' => [
            0b01000, 0b00100, 0b00010, 0b00010, 0b00010, 0b00100, 0b01000,
        ],
        '[' => [
            0b01110, 0b01000, 0b01000, 0b01000, 0b01000, 0b01000, 0b01110,
        ],
        ']' => [
            0b01110, 0b00010, 0b00010, 0b00010, 0b00010, 0b00010, 0b01110,
        ],
        '+' => [0, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0],
        '=' => [0, 0, 0b11111, 0, 0b11111, 0, 0],
        '|' => [
            0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        ' ' => [0; 7],
        _ => [0b01110, 0b10001, 0b00010, 0b00100, 0b00100, 0, 0b00100],
    }
}

#[cfg(test)]
mod tests {
    use crate::Diagnostics;

    #[test]
    fn rasterized_image_has_exact_rgba_length_and_visible_pixels() {
        let mut diagnostics = Diagnostics::default();
        diagnostics.toggle();
        let overlay = diagnostics.overlay().expect("overlay is visible");

        assert_eq!(
            overlay.rgba8.len(),
            overlay.width as usize * overlay.height as usize * 4
        );
        assert!(
            overlay
                .rgba8
                .as_chunks::<4>()
                .0
                .iter()
                .any(|pixel| pixel[3] > 0)
        );
    }
}
