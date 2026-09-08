//! Bounded Scout text from observed scanner values. No detection or clock policy.

#[derive(Clone, Copy, Debug)]
pub struct ScannerReadout<'a> {
    pub powered: bool,
    pub selected: usize,
    pub contacts: usize,
    pub label: &'a str,
    pub center_meters: Option<f64>,
    pub age_seconds: Option<f64>,
}

impl ScannerReadout<'_> {
    /// At most 39 cells per row, including malformed/very large DTO values.
    /// The existing action bar owns the shared face, contrast and DPI scaling.
    pub fn text(self) -> String {
        if !self.powered {
            return "SCANNER OFFLINE\nCORE POWER REQUIRED\n[P] CHART REQUIRES POWER".into();
        }
        if self.contacts == 0 {
            return "SCANNER - NO FRESH CONTACTS\nAUTOMATIC 5 s PULSES\n[P] CHART | LOG RANGE"
                .into();
        }
        let label: String = self
            .label
            .chars()
            .take(12)
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == ' ' {
                    c.to_ascii_uppercase()
                } else {
                    '?'
                }
            })
            .collect();
        format!(
            "SCANNER {}/{} FRESH {}\nCENTER {} km | AGE {} s\n[M] NEXT [P] CHART | LOG RANGE",
            count(self.selected.saturating_add(1)),
            count(self.contacts),
            label,
            reading(self.center_meters.map(|m| m / 1000.)),
            reading(self.age_seconds),
        )
    }
}

fn count(value: usize) -> String {
    if value > 9999 {
        ">9999".into()
    } else {
        value.to_string()
    }
}

fn reading(value: Option<f64>) -> String {
    let Some(value) = value.filter(|v| v.is_finite() && *v >= 0.) else {
        return "--".into();
    };
    let fixed = format!("{value:.1}");
    if fixed.len() <= 7 {
        fixed
    } else {
        format!("{value:.1e}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scanner_rows_bound_numbers_labels_units_and_noncolor_states() {
        let mut dto = ScannerReadout {
            powered: true,
            selected: 0,
            contacts: 1,
            label: "Vesper",
            center_meters: Some(6_400_000.),
            age_seconds: Some(5.),
        };
        let text = dto.text();
        assert!(text.contains("1/1 FRESH VESPER"));
        assert!(text.contains("CENTER 6400.0 km | AGE 5.0 s"));
        assert!(text.contains("[M] NEXT [P] CHART"));
        dto.label = "A VERY LONG UNTRUSTED NAME\nMORE";
        dto.contacts = usize::MAX;
        dto.selected = usize::MAX;
        dto.center_meters = Some(f64::MAX);
        dto.age_seconds = Some(f64::INFINITY);
        for line in dto.text().lines() {
            assert!(line.len() <= 39, "{line}");
        }
        assert!(dto.text().contains("AGE -- s"));
        dto.center_meters = Some(-1.);
        assert!(dto.text().contains("CENTER -- km"));
        dto.contacts = 0;
        assert!(dto.text().contains("NO FRESH CONTACTS"));
        dto.powered = false;
        assert!(dto.text().contains("OFFLINE\nCORE POWER REQUIRED"));
    }
}
