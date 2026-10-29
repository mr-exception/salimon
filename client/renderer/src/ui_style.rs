//! Shared Scout bitmap face and functional color vocabulary. No gameplay policy.

pub const BACKGROUND: [u8; 4] = [6, 17, 23, 255];
pub const INSET: [u8; 4] = [10, 27, 34, 255];
pub const RULE: [u8; 4] = [34, 70, 78, 255];
pub const MUTED: [u8; 4] = [132, 168, 176, 255];
pub const IVORY: [u8; 4] = [236, 243, 232, 255];
pub const CYAN: [u8; 4] = [89, 224, 215, 255];
pub const AMBER: [u8; 4] = [245, 185, 98, 255];

/// Functional palette for the linear-color 3D instance pass.
pub fn scene_ink(ink: [u8; 4]) -> [f32; 4] {
    std::array::from_fn(|i| {
        let v = f32::from(ink[i]) / 255.;
        if i == 3 || v <= 0.04045 {
            if i == 3 { v } else { v / 12.92 }
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    })
}

/// Five by seven cells. Callers may uppercase labels; metric readings retain m/k/s/g.
pub fn glyph_rows(character: char) -> [u8; 7] {
    // Deliberately preserve the case of metric prefixes: m/s and Mm/s denote
    // different scales. Labels elsewhere use the uppercase instrument face.
    match character {
        'm' => [0, 0, 0b11010, 0b10101, 0b10101, 0b10101, 0b10101],
        'k' => [
            0b10000, 0b10000, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010,
        ],
        'g' => [0, 0, 15, 17, 15, 1, 14],
        's' => [0, 0, 0b01111, 0b10000, 0b01110, 0b00001, 0b11110],
        _ => uppercase_glyph(character.to_ascii_uppercase()),
    }
}

fn uppercase_glyph(character: char) -> [u8; 7] {
    match character {
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
        ':' => [0, 4, 4, 0, 4, 4, 0],
        '.' => [0, 0, 0, 0, 0, 6, 6],
        '-' => [0, 0, 0, 31, 0, 0, 0],
        '/' => [1, 2, 2, 4, 8, 8, 16],
        '%' => [25, 26, 4, 8, 22, 6, 0],
        '>' => [16, 8, 4, 2, 4, 8, 16],
        ',' => [0, 0, 0, 0, 6, 4, 8],
        '+' => [0, 4, 4, 31, 4, 4, 0],
        '[' => [14, 8, 8, 8, 8, 8, 14],
        ']' => [14, 2, 2, 2, 2, 2, 14],
        '#' => [10, 31, 10, 10, 31, 10, 0],
        '<' => [1, 2, 4, 8, 4, 2, 1],
        '(' => [2, 4, 8, 8, 8, 4, 2],
        ')' => [8, 4, 2, 2, 2, 4, 8],
        '=' => [0, 0, 31, 0, 31, 0, 0],
        '_' => [0, 0, 0, 0, 0, 0, 31],
        '|' => [4; 7],
        '?' => [14, 17, 1, 2, 4, 0, 4],
        _ => [0; 7],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controls_and_readings_have_visible_punctuation_and_distinct_si_prefixes() {
        for ch in "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789[]<>+-/%:.,#()=_|?mksg".chars() {
            assert_ne!(glyph_rows(ch), [0; 7], "missing {ch}");
        }
        assert_ne!(glyph_rows('m'), glyph_rows('M'));
        assert_ne!(glyph_rows('k'), glyph_rows('K'));
        assert_ne!(glyph_rows('s'), glyph_rows('S'));
        for ch in "abcdefghijklmnopqrstuvwxyz"
            .chars()
            .filter(|ch| !"mksg".contains(*ch))
        {
            assert_eq!(glyph_rows(ch), glyph_rows(ch.to_ascii_uppercase()));
        }
    }

    #[test]
    fn functional_inks_have_readable_srgb_contrast_on_panel_backgrounds() {
        fn luminance(c: [u8; 4]) -> f64 {
            let linear = |v: u8| {
                let v = f64::from(v) / 255.0;
                if v <= 0.04045 {
                    v / 12.92
                } else {
                    ((v + 0.055) / 1.055).powf(2.4)
                }
            };
            0.2126 * linear(c[0]) + 0.7152 * linear(c[1]) + 0.0722 * linear(c[2])
        }
        for ink in [IVORY, MUTED, CYAN, AMBER] {
            for back in [BACKGROUND, INSET] {
                assert!((luminance(ink) + 0.05) / (luminance(back) + 0.05) >= 4.5);
            }
        }
    }
}
