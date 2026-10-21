//! Small, persistent instrument atlas for the cockpit's single wide physical display.
//!
//! The atlas changes only when a displayed value changes. All drawing is CPU-side
//! into one allocation; the renderer can skip both rasterization and GPU uploads
//! for unchanged frames.

const SECTION_WIDTH: u32 = 512;
pub(crate) const ATLAS_WIDTH: u32 = SECTION_WIDTH * 5;
pub(crate) const ATLAS_HEIGHT: u32 = 256;

/// Renderer-neutral values shown on the ship's physical cockpit monitors.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CockpitInstruments {
    pub power_off: bool,
    pub engines_off: bool,
    /// Observation only: idle, working, no power, or blocked tray.
    pub fabricator: FabricatorInstruments,
    pub speed_meters_per_second: f64,
    pub thruster_percentage: i8,
    pub core_energy_capacity_joules: u64,
    pub core_energy_stored_joules: u64,
    pub nearby_body: Option<NearbyBodyInstruments>,
    pub core_status: CoreInstruments,
    pub navigation: NavigationInstruments,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FabricatorInstruments {
    pub status: u8,
    pub jobs: u32,
    pub remaining_seconds: u32,
}

/// Renderer-only navigation values; the ship owns braking feasibility.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NavigationInstruments {
    pub direction_local: Option<[f64; 3]>,
    pub angle_degrees: Option<f64>,
    pub acceleration: Option<f64>,
    pub mass_kg: f64,
    pub relative_speed: Option<f64>,
    pub braking: Option<[f64; 3]>, // seconds, meters, joules
    pub braking_status: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct NavigationDisplay {
    flight: [String; 3],
    brake: [String; 3],
    relative_speed: String,
}
fn reading(value: Option<f64>) -> String {
    value
        .filter(|v| v.is_finite() && v.abs() < 1e12)
        .map_or_else(|| "--".to_owned(), |v| format!("{v:.1}"))
}
impl From<NavigationInstruments> for NavigationDisplay {
    fn from(v: NavigationInstruments) -> Self {
        Self {
            flight: [
                format!(
                    "V XYZ {} {} {}",
                    reading(v.direction_local.map(|d| d[0])),
                    reading(v.direction_local.map(|d| d[1])),
                    reading(v.direction_local.map(|d| d[2]))
                ),
                format!(
                    "ANGLE {} DEG  A {} m/s2",
                    reading(v.angle_degrees),
                    reading(v.acceleration)
                ),
                format!("MASS {} kg", reading(Some(v.mass_kg))),
            ],
            brake: [
                format!("REVERSE {}", v.braking_status),
                format!(
                    "T {} s  D {} m",
                    reading(v.braking.map(|b| b[0])),
                    reading(v.braking.map(|b| b[1]))
                ),
                format!("ENERGY {} GJ", reading(v.braking.map(|b| b[2] / 1e9))),
            ],
            relative_speed: format!("TOTAL {} m/s", reading(v.relative_speed)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NearbyBodyInstruments {
    pub name: &'static str,
    pub surface_distance_meters: f64,
    /// Negative approaches, positive recedes, and zero is stationary.
    pub radial_speed_meters_per_second: f64,
}

/// Values for the distinct physical Core display; power comes from the bus DTO.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CoreInstruments {
    pub consumption_watts: f64,
    pub conversion_watts: f64,
    pub conversion_remaining_joules: f64,
    pub depletion_seconds: Option<f64>,
    pub low_energy: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CoreDisplay {
    consumption_kw: u64,
    conversion_kw: u64,
    remaining: EnergyDisplay,
    depletion_seconds: Option<u64>,
    converting: bool,
    low_energy: bool,
}
impl From<CoreInstruments> for CoreDisplay {
    fn from(v: CoreInstruments) -> Self {
        Self {
            consumption_kw: (v.consumption_watts.max(0.0) / 1000.0).round() as u64,
            conversion_kw: (v.conversion_watts.max(0.0) / 1000.0).round() as u64,
            remaining: EnergyDisplay::from_joules(v.conversion_remaining_joules.max(0.0) as u64),
            depletion_seconds: v
                .depletion_seconds
                .filter(|s| s.is_finite() && *s >= 0.0)
                .map(|s| s.ceil() as u64),
            converting: v.conversion_remaining_joules > 0.0,
            low_energy: v.low_energy,
        }
    }
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

#[derive(Clone, Debug, PartialEq, Eq)]
struct DisplayKey {
    power_off: bool,
    engines_off: bool,
    fabricator: FabricatorInstruments,
    speed: SpeedDisplay,
    power: i8,
    core_capacity: EnergyDisplay,
    core_stored: EnergyDisplay,
    nearby: Option<NearbyDisplay>,
    core_status: CoreDisplay,
    navigation: NavigationDisplay,
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
            power_off: values.power_off,
            fabricator: values.fabricator,
            speed: SpeedDisplay::from_meters_per_second(values.speed_meters_per_second),
            power: values.thruster_percentage.clamp(-100, 100),
            engines_off: values.engines_off,
            core_capacity: EnergyDisplay::from_joules(values.core_energy_capacity_joules),
            core_stored: EnergyDisplay::from_joules(values.core_energy_stored_joules),
            nearby: values.nearby_body.map(NearbyDisplay::from_instruments),
            core_status: values.core_status.into(),
            navigation: values.navigation.into(),
        };
        if next.power_off {
            if self
                .displayed
                .as_ref()
                .is_some_and(|previous| previous.power_off)
            {
                return false;
            }
            for pixel in self.pixels.as_chunks_mut::<4>().0 {
                pixel.copy_from_slice(&[0, 0, 0, 255]);
            }
            self.displayed = Some(next);
            return true;
        }
        if self.displayed.as_ref() == Some(&next) {
            return false;
        }

        // The flight region includes the shared power command once. Independent
        // energy/navigation changes leave the other regions untouched.
        if self.displayed.as_ref().is_none_or(|previous| {
            previous.power_off
                || previous.speed != next.speed
                || previous.power != next.power
                || previous.engines_off != next.engines_off
                || previous.navigation.flight != next.navigation.flight
        }) {
            self.draw_speed(next.speed, next.power, next.engines_off, &next.navigation);
        }
        if self.displayed.as_ref().is_none_or(|previous| {
            previous.power_off
                || previous.core_capacity != next.core_capacity
                || previous.core_stored != next.core_stored
                || previous.navigation.brake != next.navigation.brake
        }) {
            self.draw_core(next.core_stored, next.core_capacity, &next.navigation);
        }
        if self.displayed.as_ref().is_none_or(|previous| {
            previous.power_off
                || previous.nearby != next.nearby
                || previous.navigation.relative_speed != next.navigation.relative_speed
        }) {
            self.draw_nearby(next.nearby);
            self.text(28, 222, &next.navigation.relative_speed, 2, CYAN);
        }
        if self.displayed.as_ref().is_none_or(|previous| {
            previous.power_off
                || previous.core_status != next.core_status
                || previous.core_capacity != next.core_capacity
                || previous.core_stored != next.core_stored
        }) {
            self.draw_core_status(&next);
        }
        self.section_frame(4, "FABRICATOR");
        self.text(
            28,
            70,
            match next.fabricator.status {
                1 => "WORKING",
                2 => "PAUSED / NO POWER",
                3 => "OUTPUT BLOCKED",
                _ => "IDLE",
            },
            3,
            CYAN,
        );
        self.text(
            28,
            112,
            &format!(
                "JOBS {} / LEFT {}s",
                next.fabricator.jobs, next.fabricator.remaining_seconds
            ),
            2,
            IVORY,
        );
        self.text(28, 160, "E AT SCREEN TO CRAFT", 2, IVORY);
        self.text(28, 192, "AIM AT PORT TO INSERT", 2, MUTED);
        self.displayed = Some(next);
        true
    }

    fn draw_core_status(&mut self, key: &DisplayKey) {
        self.section_frame(3, "ENERGY CORE");
        let status = key.core_status;
        let color = if status.low_energy { AMBER } else { CYAN };
        self.text(
            28,
            66,
            &format!(
                "{} {} / {} {}",
                key.core_stored.number(),
                key.core_stored.unit.label(),
                key.core_capacity.number(),
                key.core_capacity.unit.label()
            ),
            3,
            IVORY,
        );
        self.text(
            28,
            98,
            &format!("DRAW {} kW", status.consumption_kw),
            3,
            CYAN,
        );
        self.text(
            28,
            132,
            if status.converting {
                "CONVERTING"
            } else {
                "INTAKE READY"
            },
            3,
            CYAN,
        );
        self.text(
            28,
            159,
            &format!(
                "{} kW  {} {} LEFT",
                status.conversion_kw,
                status.remaining.number(),
                status.remaining.unit.label()
            ),
            2,
            MUTED,
        );
        let time = status
            .depletion_seconds
            .map_or("--".to_owned(), |s| format!("{s}s"));
        self.text(28, 190, &format!("DEPLETION {time}"), 3, color);
        self.text(
            28,
            226,
            if status.low_energy {
                "LOW ENERGY - REFUEL"
            } else {
                "POWER AVAILABLE"
            },
            2,
            color,
        );
    }

    fn draw_speed(
        &mut self,
        speed: SpeedDisplay,
        percentage: i8,
        engines_off: bool,
        nav: &NavigationDisplay,
    ) {
        self.section_frame(0, "INERTIAL FLIGHT");
        let (number, unit, color) = speed_text(speed);
        self.text(28, 66, &format!("{number} {unit}"), 4, color);
        self.text(
            28,
            112,
            &format!(
                "{percentage}% / ENG {} [B]",
                if engines_off { "OFF" } else { "ON" }
            ),
            3,
            CYAN,
        );
        for (i, row) in nav.flight.iter().enumerate() {
            self.text(28, 153 + i as u32 * 28, row, 2, IVORY);
        }
    }

    fn draw_core(
        &mut self,
        stored: EnergyDisplay,
        capacity: EnergyDisplay,
        nav: &NavigationDisplay,
    ) {
        self.section_frame(1, "CORE / BRAKING");
        self.text(
            28,
            70,
            &format!(
                "{} {} / {} {}",
                stored.number(),
                stored.unit.label(),
                capacity.number(),
                capacity.unit.label()
            ),
            2,
            IVORY,
        );
        for (i, row) in nav.brake.iter().enumerate() {
            self.text(28, 115 + i as u32 * 32, row, 2, CYAN);
        }
        self.text(28, 225, "FIXED HEADING - NO AUTOBRAKE", 2, MUTED);
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

    pub(super) fn regions(pixels: &[u8]) -> Vec<u8> {
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
        for percentage in [-100, -1, 0, 1, 100] {
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
            thruster_percentage: i8::MAX,
            ..CockpitInstruments::default()
        }));
        assert!(atlas.update(CockpitInstruments {
            speed_meters_per_second: f64::NAN,
            thruster_percentage: i8::MIN,
            ..CockpitInstruments::default()
        }));
        assert_eq!(atlas.displayed.as_ref().unwrap().power, -100);
        assert!(!atlas.update(CockpitInstruments {
            speed_meters_per_second: f64::NAN,
            thruster_percentage: -100,
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
            regions(&atlas.pixels)[panel_bytes * 2..panel_bytes * 3],
            original[panel_bytes * 2..panel_bytes * 3]
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
        assert_eq!(
            atlas.pixels.len(),
            ATLAS_WIDTH as usize * ATLAS_HEIGHT as usize * 4
        );
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

#[cfg(test)]
mod power_tests {
    use super::*;
    #[test]
    fn power_loss_blanks_once_and_restoration_redraws_all_regions() {
        let mut atlas = InstrumentAtlas::new();
        let values = CockpitInstruments {
            core_energy_capacity_joules: 60_000_000_000,
            core_energy_stored_joules: 45_000_000_000,
            ..Default::default()
        };
        atlas.update(values);
        let before = atlas.pixels.clone();
        assert!(atlas.update(CockpitInstruments {
            power_off: true,
            ..values
        }));
        assert!(
            atlas
                .pixels
                .as_chunks::<4>()
                .0
                .iter()
                .all(|pixel| *pixel == [0, 0, 0, 255])
        );
        assert!(!atlas.update(CockpitInstruments {
            power_off: true,
            speed_meters_per_second: 100.0,
            ..values
        }));
        assert!(atlas.update(values));
        assert_eq!(atlas.pixels, before);
    }
}

#[cfg(test)]
mod core_display_tests {
    use super::*;
    fn core_region(atlas: &InstrumentAtlas) -> Vec<u8> {
        (0..ATLAS_HEIGHT)
            .flat_map(|y| {
                let start = ((y * ATLAS_WIDTH + 3 * SECTION_WIDTH) * 4) as usize;
                atlas.pixels[start..start + (SECTION_WIDTH * 4) as usize]
                    .iter()
                    .copied()
            })
            .collect()
    }
    #[test]
    fn core_changes_preserve_console_and_cache_visible_precision() {
        let mut atlas = InstrumentAtlas::new();
        let base = CockpitInstruments::default();
        atlas.update(base);
        let console = super::tests::regions(&atlas.pixels);
        let before = core_region(&atlas);
        let values = CockpitInstruments {
            core_status: CoreInstruments {
                consumption_watts: 100_000_000.0,
                conversion_watts: 200_000_000.0,
                conversion_remaining_joules: 1_000_000_000.0,
                depletion_seconds: Some(100.0),
                low_energy: true,
            },
            ..base
        };
        assert!(atlas.update(values));
        assert_eq!(super::tests::regions(&atlas.pixels), console);
        assert_ne!(core_region(&atlas), before);
        assert!(!atlas.update(CockpitInstruments {
            core_status: CoreInstruments {
                consumption_watts: 100_000_001.0,
                ..values.core_status
            },
            ..values
        }));
        assert!(!atlas.update(values));
    }
}

#[cfg(test)]
mod navigation_display_tests {
    use super::*;
    #[test]
    fn rounded_navigation_updates_owning_regions_and_rejects_nonfinite_readings() {
        let mut atlas = InstrumentAtlas::new();
        let base = CockpitInstruments::default();
        atlas.update(base);
        let before = super::tests::regions(&atlas.pixels);
        let v = CockpitInstruments {
            navigation: NavigationInstruments {
                mass_kg: 100_000.0,
                acceleration: Some(-20.0),
                direction_local: Some([1.0, 0.0, 0.0]),
                angle_degrees: Some(0.0),
                ..Default::default()
            },
            ..base
        };
        assert!(atlas.update(v));
        let after = super::tests::regions(&atlas.pixels);
        let bytes = (SECTION_WIDTH * ATLAS_HEIGHT * 4) as usize;
        assert_ne!(before[..bytes], after[..bytes]);
        assert_eq!(before[bytes..], after[bytes..]);
        assert!(!atlas.update(CockpitInstruments {
            navigation: NavigationInstruments {
                mass_kg: 100_000.01,
                ..v.navigation
            },
            ..v
        }));
        assert_eq!(reading(Some(f64::NAN)), "--");
        assert_eq!(reading(Some(f64::INFINITY)), "--");
        for row in NavigationDisplay::from(NavigationInstruments {
            braking_status: "ALIGN NOSE TO VELOCITY",
            ..v.navigation
        })
        .flight
        {
            assert!(row.len() <= 38);
        }
    }
    #[test]
    fn engine_off_is_distinct_at_zero_throttle_and_invalidates_the_flight_region() {
        let mut atlas = InstrumentAtlas::new();
        let on = CockpitInstruments::default();
        assert!(atlas.update(on));
        let before = atlas.pixels.clone();
        let off = CockpitInstruments {
            engines_off: true,
            ..on
        };
        assert!(atlas.update(off));
        assert_ne!(atlas.pixels, before);
        assert!(!atlas.update(off));
        assert!(atlas.update(on));
        assert_eq!(atlas.pixels, before);
    }
}
