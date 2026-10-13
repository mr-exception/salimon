//! Small, persistent instrument atlas for the cockpit's single wide physical display.
//!
//! The atlas changes only when a displayed value changes. All drawing is CPU-side
//! into one allocation; the renderer can skip both rasterization and GPU uploads
//! for unchanged frames.

const SECTION_WIDTH: u32 = 512;
pub(crate) const ATLAS_WIDTH: u32 = SECTION_WIDTH * 3;
pub(crate) const ATLAS_HEIGHT: u32 = 256;

/// Renderer-neutral values shown on the ship's physical cockpit monitors.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CockpitInstruments {
    pub speed_meters_per_second: f64,
    pub thruster_percentage: u8,
    pub core_energy_capacity_joules: u64,
    pub core_energy_stored_joules: u64,
    pub nearby_body: Option<NearbyBodyInstruments>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NearbyBodyInstruments {
    pub name: &'static str,
    pub surface_distance_meters: f64,
    /// Negative approaches, positive recedes, and zero is stationary.
    pub radial_speed_meters_per_second: f64,
}

const BACKGROUND: [u8; 4] = [6, 17, 23, 255];
const INSET: [u8; 4] = [10, 27, 34, 255];
const RULE: [u8; 4] = [34, 70, 78, 255];
const MUTED: [u8; 4] = [132, 168, 176, 255];
const IVORY: [u8; 4] = [236, 243, 232, 255];
const CYAN: [u8; 4] = [89, 224, 215, 255];
const AMBER: [u8; 4] = [245, 185, 98, 255];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SpeedUnit {
    Meters,
    Kilometers,
    Megameters,
}

impl SpeedUnit {
    fn precision(self) -> u32 {
        match self {
            Self::Meters => 10,
            Self::Kilometers | Self::Megameters => 100,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Meters => "m/s",
            Self::Kilometers => "km/s",
            Self::Megameters => "Mm/s",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SpeedDisplay {
    Unavailable,
    Value { minor_units: u32, unit: SpeedUnit },
    AboveRange,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EnergyUnit {
    Gigajoules,
    Terajoules,
}

impl EnergyUnit {
    fn divisor(self) -> f64 {
        match self {
            Self::Gigajoules => 1_000_000_000.0,
            Self::Terajoules => 1_000_000_000_000.0,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Gigajoules => "GJ",
            Self::Terajoules => "TJ",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct EnergyDisplay {
    hundredths: u64,
    unit: EnergyUnit,
}

impl EnergyDisplay {
    fn from_joules(joules: u64) -> Self {
        let unit = if joules >= 999_500_000_000 {
            EnergyUnit::Terajoules
        } else {
            EnergyUnit::Gigajoules
        };
        Self {
            hundredths: (joules as f64 / unit.divisor() * 100.0).round() as u64,
            unit,
        }
    }

    fn number(self) -> String {
        format!("{}.{:02}", self.hundredths / 100, self.hundredths % 100)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RadialDirection {
    Approaching,
    Receding,
    Zero,
    Unavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct NearbyDisplay {
    name: &'static str,
    distance: SpeedDisplay,
    radial_speed: SpeedDisplay,
    direction: RadialDirection,
}

impl NearbyDisplay {
    fn from_instruments(values: NearbyBodyInstruments) -> Self {
        let radial_speed =
            SpeedDisplay::from_meters_per_second(values.radial_speed_meters_per_second.abs());
        let direction = match radial_speed {
            SpeedDisplay::Value { minor_units: 0, .. } => RadialDirection::Zero,
            SpeedDisplay::Unavailable | SpeedDisplay::AboveRange => RadialDirection::Unavailable,
            SpeedDisplay::Value { .. } if values.radial_speed_meters_per_second < 0.0 => {
                RadialDirection::Approaching
            }
            SpeedDisplay::Value { .. } => RadialDirection::Receding,
        };
        Self {
            name: values.name,
            distance: SpeedDisplay::from_meters_per_second(values.surface_distance_meters),
            radial_speed,
            direction,
        }
    }
}

impl SpeedDisplay {
    fn from_meters_per_second(speed: f64) -> Self {
        if !speed.is_finite() || speed < 0.0 {
            return Self::Unavailable;
        }

        // Promote after rounding, so a display never reads "1000.0 m/s" next
        // to another frame reading "1.00 km/s" at the same visible precision.
        for (divisor, unit) in [
            (1.0, SpeedUnit::Meters),
            (1_000.0, SpeedUnit::Kilometers),
            (1_000_000.0, SpeedUnit::Megameters),
        ] {
            let minor_units = (speed / divisor * f64::from(unit.precision())).round();
            if minor_units < 1_000.0 * f64::from(unit.precision()) {
                return Self::Value {
                    minor_units: minor_units as u32,
                    unit,
                };
            }
        }
        Self::AboveRange
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DisplayKey {
    speed: SpeedDisplay,
    power: u8,
    core_capacity: EnergyDisplay,
    core_stored: EnergyDisplay,
    nearby: Option<NearbyDisplay>,
}

pub(crate) struct InstrumentAtlas {
    pub(crate) pixels: Vec<u8>,
    displayed: Option<DisplayKey>,
    section_x: u32,
}

impl InstrumentAtlas {
    pub(crate) fn new() -> Self {
        Self {
            pixels: BACKGROUND.repeat((ATLAS_WIDTH * ATLAS_HEIGHT) as usize),
            displayed: None,
            section_x: 0,
        }
    }

    /// Returns true when the caller needs to upload the atlas to the GPU.
    pub(crate) fn update(&mut self, values: CockpitInstruments) -> bool {
        let next = DisplayKey {
            speed: SpeedDisplay::from_meters_per_second(values.speed_meters_per_second),
            power: values.thruster_percentage.min(100),
            core_capacity: EnergyDisplay::from_joules(values.core_energy_capacity_joules),
            core_stored: EnergyDisplay::from_joules(values.core_energy_stored_joules),
            nearby: values.nearby_body.map(NearbyDisplay::from_instruments),
        };
        if self.displayed == Some(next) {
            return false;
        }

        // The flight region includes the shared power command once. Independent
        // energy/navigation changes leave the other regions untouched.
        if self
            .displayed
            .is_none_or(|previous| previous.speed != next.speed || previous.power != next.power)
        {
            self.draw_speed(next.speed, next.power);
        }
        if self.displayed.is_none_or(|previous| {
            previous.core_capacity != next.core_capacity || previous.core_stored != next.core_stored
        }) {
            self.draw_core(next.core_stored, next.core_capacity);
        }
        if self
            .displayed
            .is_none_or(|previous| previous.nearby != next.nearby)
        {
            self.draw_nearby(next.nearby);
        }
        self.displayed = Some(next);
        true
    }

    fn draw_speed(&mut self, speed: SpeedDisplay, percentage: u8) {
        self.section_frame(0, "SPEED");
        let (number, unit, color) = speed_text(speed);
        self.text(28, 80, &number, 10, color);
        self.text(392, 119, unit, 3, CYAN);
        self.text(
            340,
            156,
            match speed {
                SpeedDisplay::Unavailable => "NO READING",
                SpeedDisplay::AboveRange => "ABOVE RANGE",
                SpeedDisplay::Value { .. } => "METRIC UNITS",
            },
            2,
            if color == AMBER { AMBER } else { MUTED },
        );
        self.rect(28, 175, 456, 2, RULE);
        self.rect(28, 175, 48, 2, CYAN);
        self.text(28, 194, "THRUSTER POWER", 2, MUTED);
        self.text(28, 212, &format!("{percentage}%"), 4, CYAN);
        for segment in 0..20 {
            let x = 160 + segment * 15;
            self.rect(x, 212, 12, 28, RULE);
            let filled = u32::from(percentage).saturating_sub(segment * 5).min(5);
            if filled > 0 {
                self.rect(x, 212, (12 * filled).div_ceil(5), 28, CYAN);
            }
        }
    }

    fn draw_core(&mut self, stored: EnergyDisplay, capacity: EnergyDisplay) {
        self.section_frame(1, "CORE ENERGY");
        self.text(28, 70, "STORED", 2, MUTED);
        let stored_number = stored.number();
        let scale = (360 / (stored_number.len() as u32 * 6)).clamp(2, 6);
        self.text(28, 91, &stored_number, scale, IVORY);
        self.text(408, 112, stored.unit.label(), 2, CYAN);
        self.text(28, 157, "CAPACITY", 2, MUTED);
        self.text(140, 157, &capacity.number(), 3, IVORY);
        self.text(408, 157, capacity.unit.label(), 2, CYAN);
    }

    fn draw_nearby(&mut self, nearby: Option<NearbyDisplay>) {
        self.section_frame(2, "NEARBY BODY");
        match nearby {
            Some(nearby) => {
                self.text(28, 70, &nearby.name.to_ascii_uppercase(), 3, IVORY);
                let (distance, _, _) = speed_text(nearby.distance);
                self.text(28, 108, "DIST", 2, MUTED);
                self.text(110, 103, &distance, 3, IVORY);
                self.text(352, 108, distance_unit(nearby.distance), 2, CYAN);
                let (radial, radial_unit, _) = speed_text(nearby.radial_speed);
                let direction = match nearby.direction {
                    RadialDirection::Approaching => "APPROACHING",
                    RadialDirection::Receding => "RECEDING",
                    RadialDirection::Zero => "ZERO RADIAL",
                    RadialDirection::Unavailable => "NO RADIAL DATA",
                };
                self.text(
                    28,
                    150,
                    direction,
                    3,
                    match nearby.direction {
                        RadialDirection::Approaching => CYAN,
                        RadialDirection::Receding => AMBER,
                        _ => MUTED,
                    },
                );
                self.text(28, 177, &radial, 3, IVORY);
                self.text(220, 177, radial_unit, 3, MUTED);
            }
            None => {
                self.text(28, 83, "NO TARGET", 5, AMBER);
                self.text(28, 133, "OUT OF 3.00 Mm RANGE", 2, MUTED);
                self.text(28, 169, "DIST --   RADIAL --", 2, MUTED);
            }
        }
    }

    fn section_frame(&mut self, section: u32, title: &str) {
        self.section_x = section * SECTION_WIDTH;
        self.rect(0, 0, SECTION_WIDTH, ATLAS_HEIGHT, INSET);
        if section > 0 {
            self.rect(0, 20, 2, 216, RULE);
        }
        self.text(28, 25, title, 3, IVORY);
        self.rect(423, 28, 6, 6, CYAN);
        self.text(432, 28, "LIVE", 2, CYAN);
        self.rect(28, 51, 456, 1, RULE);
        self.rect(28, 50, 52, 3, CYAN);
    }

    fn rect(&mut self, x: u32, y: u32, width: u32, height: u32, color: [u8; 4]) {
        let x = self.section_x + x;
        for row in y.min(ATLAS_HEIGHT)..y.saturating_add(height).min(ATLAS_HEIGHT) {
            for column in
                x.min(ATLAS_WIDTH)..x.saturating_add(width).min(self.section_x + SECTION_WIDTH)
            {
                let offset = ((row * ATLAS_WIDTH + column) * 4) as usize;
                self.pixels[offset..offset + 4].copy_from_slice(&color);
            }
        }
    }

    fn text(&mut self, x: u32, y: u32, text: &str, scale: u32, color: [u8; 4]) {
        for (index, character) in text.chars().enumerate() {
            for (row, bits) in glyph_rows(character).into_iter().enumerate() {
                for column in 0..5 {
                    if bits & (1 << (4 - column)) != 0 {
                        self.rect(
                            x + (index as u32 * 6 + column) * scale,
                            y + row as u32 * scale,
                            scale,
                            scale,
                            color,
                        );
                    }
                }
            }
        }
    }
}

fn speed_text(speed: SpeedDisplay) -> (String, &'static str, [u8; 4]) {
    match speed {
        SpeedDisplay::Unavailable => ("--".to_owned(), "m/s", AMBER),
        SpeedDisplay::Value { minor_units, unit } => {
            let precision = unit.precision();
            let decimals = if precision == 10 { 1 } else { 2 };
            (
                format!(
                    "{}.{:0decimals$}",
                    minor_units / precision,
                    minor_units % precision,
                ),
                unit.label(),
                IVORY,
            )
        }
        SpeedDisplay::AboveRange => (">999".to_owned(), "Mm/s", AMBER),
    }
}

fn distance_unit(distance: SpeedDisplay) -> &'static str {
    match distance {
        SpeedDisplay::Value {
            unit: SpeedUnit::Meters,
            ..
        }
        | SpeedDisplay::Unavailable => "m",
        SpeedDisplay::Value {
            unit: SpeedUnit::Kilometers,
            ..
        } => "km",
        _ => "Mm",
    }
}

pub(crate) fn glyph_rows(character: char) -> [u8; 7] {
    // Deliberately preserve the case of metric prefixes: m/s and Mm/s denote
    // different scales. Labels elsewhere use the uppercase instrument face.
    match character {
        'm' => [0, 0, 0b11010, 0b10101, 0b10101, 0b10101, 0b10101],
        'k' => [
            0b10000, 0b10000, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010,
        ],
        's' => [0, 0, 0b01111, 0b10000, 0b01110, 0b00001, 0b11110],
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
        _ => [0; 7],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn regions(pixels: &[u8]) -> Vec<u8> {
        (0..3)
            .flat_map(|section| {
                (0..ATLAS_HEIGHT).flat_map(move |y| {
                    let start = ((y * ATLAS_WIDTH + section * SECTION_WIDTH) * 4) as usize;
                    pixels[start..start + (SECTION_WIDTH * 4) as usize]
                        .iter()
                        .copied()
                })
            })
            .collect()
    }

    #[test]
    fn speed_rounds_and_promotes_metric_units_without_long_digits() {
        for (speed, minor_units, unit) in [
            (0.0, 0, SpeedUnit::Meters),
            (999.94, 9999, SpeedUnit::Meters),
            (999.95, 100, SpeedUnit::Kilometers),
            (1_000.0, 100, SpeedUnit::Kilometers),
            (999_994.0, 99999, SpeedUnit::Kilometers),
            (999_995.0, 100, SpeedUnit::Megameters),
            (2_500_000.0, 250, SpeedUnit::Megameters),
        ] {
            assert_eq!(
                SpeedDisplay::from_meters_per_second(speed),
                SpeedDisplay::Value { minor_units, unit },
            );
        }
        assert_ne!(glyph_rows('m'), glyph_rows('M'));
    }

    #[test]
    fn invalid_and_extreme_speeds_have_bounded_readings() {
        for speed in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                SpeedDisplay::from_meters_per_second(speed),
                SpeedDisplay::Unavailable,
            );
        }
        assert_eq!(
            SpeedDisplay::from_meters_per_second(f64::MAX),
            SpeedDisplay::AboveRange,
        );
    }

    #[test]
    fn unchanged_visible_values_preserve_atlas_and_allocation() {
        let mut atlas = InstrumentAtlas::new();
        let values = CockpitInstruments {
            speed_meters_per_second: 10.01,
            thruster_percentage: 32,
            ..CockpitInstruments::default()
        };
        let allocation = atlas.pixels.as_ptr();
        assert!(atlas.update(values));
        let original = regions(&atlas.pixels);
        assert!(!atlas.update(values));
        assert!(!atlas.update(CockpitInstruments {
            speed_meters_per_second: 10.04,
            ..values
        }));
        assert_eq!(regions(&atlas.pixels), original);
        assert_eq!(atlas.pixels.as_ptr(), allocation);
        assert!(atlas.update(CockpitInstruments {
            speed_meters_per_second: 10.06,
            ..values
        }));
        let panel_bytes = (SECTION_WIDTH * ATLAS_HEIGHT * 4) as usize;
        assert_ne!(
            regions(&atlas.pixels)[..panel_bytes],
            original[..panel_bytes]
        );
        assert_eq!(
            regions(&atlas.pixels)[panel_bytes..],
            original[panel_bytes..]
        );
    }

    #[test]
    fn changed_power_updates_only_flight_region() {
        let mut atlas = InstrumentAtlas::new();
        assert!(atlas.update(CockpitInstruments::default()));
        let original = regions(&atlas.pixels);
        assert!(atlas.update(CockpitInstruments {
            thruster_percentage: 1,
            ..CockpitInstruments::default()
        }));
        let panel_bytes = (SECTION_WIDTH * ATLAS_HEIGHT * 4) as usize;
        assert_ne!(
            regions(&atlas.pixels)[..panel_bytes],
            original[..panel_bytes]
        );
        assert_eq!(
            regions(&atlas.pixels)[panel_bytes..],
            original[panel_bytes..]
        );
    }

    #[test]
    fn power_clamps_and_all_pixels_remain_opaque() {
        let mut atlas = InstrumentAtlas::new();
        for percentage in [0, 1, 100] {
            assert!(atlas.update(CockpitInstruments {
                speed_meters_per_second: f64::NAN,
                thruster_percentage: percentage,
                ..CockpitInstruments::default()
            }));
            assert!(
                atlas
                    .pixels
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .all(|pixel| pixel[3] == 255)
            );
        }
        assert!(!atlas.update(CockpitInstruments {
            speed_meters_per_second: f64::NAN,
            thruster_percentage: u8::MAX,
            ..CockpitInstruments::default()
        }));
    }

    #[test]
    fn energy_only_changes_the_energy_region_at_visible_precision() {
        let mut atlas = InstrumentAtlas::new();
        let values = CockpitInstruments {
            core_energy_capacity_joules: 1_000_000_000_000,
            core_energy_stored_joules: 750_000_000_000,
            ..CockpitInstruments::default()
        };
        assert!(atlas.update(values));
        let original = regions(&atlas.pixels);
        assert!(!atlas.update(CockpitInstruments {
            core_energy_stored_joules: values.core_energy_stored_joules + 1,
            ..values
        }));
        assert!(atlas.update(CockpitInstruments {
            core_energy_stored_joules: 500_000_000_000,
            ..values
        }));
        let panel_bytes = (SECTION_WIDTH * ATLAS_HEIGHT * 4) as usize;
        assert_eq!(
            regions(&atlas.pixels)[..panel_bytes],
            original[..panel_bytes]
        );
        assert_ne!(
            regions(&atlas.pixels)[panel_bytes..panel_bytes * 2],
            original[panel_bytes..panel_bytes * 2]
        );
        assert_eq!(
            regions(&atlas.pixels)[panel_bytes * 2..],
            original[panel_bytes * 2..]
        );
    }

    #[test]
    fn nearby_target_and_radial_direction_only_change_navigation_region() {
        let mut atlas = InstrumentAtlas::new();
        let values = CockpitInstruments {
            nearby_body: Some(NearbyBodyInstruments {
                name: "Earth",
                surface_distance_meters: 2_000.0,
                radial_speed_meters_per_second: -25.0,
            }),
            ..CockpitInstruments::default()
        };
        assert!(atlas.update(values));
        let approaching = regions(&atlas.pixels);
        assert!(atlas.update(CockpitInstruments {
            nearby_body: Some(NearbyBodyInstruments {
                radial_speed_meters_per_second: 25.0,
                ..values.nearby_body.unwrap()
            }),
            ..values
        }));
        let receding = regions(&atlas.pixels);
        let panel_bytes = (SECTION_WIDTH * ATLAS_HEIGHT * 4) as usize;
        assert_eq!(approaching[..panel_bytes * 2], receding[..panel_bytes * 2]);
        assert_ne!(approaching[panel_bytes * 2..], receding[panel_bytes * 2..]);

        assert!(atlas.update(CockpitInstruments {
            nearby_body: None,
            ..values
        }));
        assert_ne!(
            regions(&atlas.pixels)[panel_bytes * 2..],
            receding[panel_bytes * 2..]
        );
    }

    #[test]
    fn capacity_and_navigation_fields_invalidate_only_their_region() {
        let mut atlas = InstrumentAtlas::new();
        let base = CockpitInstruments {
            core_energy_capacity_joules: 1_000_000_000_000,
            nearby_body: Some(NearbyBodyInstruments {
                name: "Earth",
                surface_distance_meters: 0.0,
                radial_speed_meters_per_second: 0.0,
            }),
            ..CockpitInstruments::default()
        };
        assert!(atlas.update(base));
        let bytes = (SECTION_WIDTH * ATLAS_HEIGHT * 4) as usize;
        for (values, changed) in [
            (
                CockpitInstruments {
                    core_energy_capacity_joules: u64::MAX,
                    ..base
                },
                1,
            ),
            (
                CockpitInstruments {
                    nearby_body: Some(NearbyBodyInstruments {
                        name: "Mars",
                        ..base.nearby_body.unwrap()
                    }),
                    ..base
                },
                2,
            ),
            (
                CockpitInstruments {
                    nearby_body: Some(NearbyBodyInstruments {
                        surface_distance_meters: f64::NAN,
                        ..base.nearby_body.unwrap()
                    }),
                    ..base
                },
                2,
            ),
            (
                CockpitInstruments {
                    nearby_body: Some(NearbyBodyInstruments {
                        radial_speed_meters_per_second: f64::INFINITY,
                        ..base.nearby_body.unwrap()
                    }),
                    ..base
                },
                2,
            ),
        ] {
            atlas.update(base);
            let before = regions(&atlas.pixels);
            assert!(atlas.update(values));
            let after = regions(&atlas.pixels);
            for section in 0..3 {
                let range = section * bytes..(section + 1) * bytes;
                assert_eq!(before[range.clone()] == after[range], section != changed);
            }
            assert!(!atlas.update(values));
        }
        assert_eq!(atlas.pixels.len(), 1536 * 256 * 4);
        assert_eq!(
            distance_unit(SpeedDisplay::from_meters_per_second(0.0)),
            "m"
        );
        assert_eq!(
            distance_unit(SpeedDisplay::from_meters_per_second(2000.0)),
            "km"
        );
        assert_eq!(
            distance_unit(SpeedDisplay::from_meters_per_second(2_000_000.0)),
            "Mm"
        );
    }

    #[test]
    fn radial_direction_is_classified_after_visible_rounding() {
        let base = NearbyBodyInstruments {
            name: "Mars",
            surface_distance_meters: 1.0,
            radial_speed_meters_per_second: -0.04,
        };
        assert_eq!(
            NearbyDisplay::from_instruments(base).direction,
            RadialDirection::Zero
        );
        assert_eq!(
            NearbyDisplay::from_instruments(NearbyBodyInstruments {
                radial_speed_meters_per_second: -0.06,
                ..base
            })
            .direction,
            RadialDirection::Approaching
        );
        assert_eq!(
            NearbyDisplay::from_instruments(NearbyBodyInstruments {
                radial_speed_meters_per_second: 0.06,
                ..base
            })
            .direction,
            RadialDirection::Receding
        );
    }
}
