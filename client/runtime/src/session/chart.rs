//! Player-known navigation composition. No catalog search/reveal or flight command.
use super::{
    GameplaySession,
    scanner::{CATALOG, Scanner},
};
use salimon_persistence::{SaveError, invalid};
use salimon_universe::scanner::{
    Position,
    chart::{Chart, ChartCheckpoint},
};

pub(crate) fn validate_saved(source: &ChartCheckpoint) -> Result<(), SaveError> {
    if source.catalog_revision != CATALOG.revision() {
        return Err(SaveError::IncompatibleCatalog(
            "player chart catalog revision".into(),
        ));
    }
    Chart::from_checkpoint(&CATALOG, source.clone()).map_err(|e| invalid(e.to_string()))?;
    Ok(())
}
pub(crate) fn restore(
    source: Option<ChartCheckpoint>,
    schema: u32,
    canonical: bool,
    scanner: &Scanner,
) -> Result<Chart, SaveError> {
    let chart = match (canonical, schema, source) {
        (true, 26.., Some(source)) => {
            validate_saved(&source)?;
            Chart::from_checkpoint(&CATALOG, source).map_err(|e| invalid(e.to_string()))?
        }
        (true, 1..=25, None) => {
            let mut c = Chart::new(CATALOG.revision()).map_err(|e| invalid(e.to_string()))?;
            c.observe(&scanner.contacts)
                .map_err(|e| invalid(e.to_string()))?;
            c
        }
        (false, _, None) => Chart::new(CATALOG.revision()).map_err(|e| invalid(e.to_string()))?,
        _ => {
            return Err(invalid(
                "chart source missing or incompatible with schema/world",
            ));
        }
    };
    let known: std::collections::BTreeMap<_, _> = scanner
        .contacts
        .history()
        .map(|e| (&e.observation().id, e))
        .collect();
    for e in scanner.contacts.history() {
        if chart
            .get(&e.observation().id)
            .and_then(|entry| entry.observed())
            != Some(e)
        {
            return Err(invalid("chart and scanner discovery disagree"));
        }
    }
    for entry in chart.entries() {
        if let Some(e) = entry.observed()
            && known.get(&entry.id).copied() != Some(e)
        {
            return Err(invalid("chart contains unobserved verified knowledge"));
        }
        if entry
            .leads()
            .iter()
            .any(|lead| lead.received_utc_millis > scanner.utc_millis)
        {
            return Err(invalid("chart lead timestamp ahead of clock"));
        }
    }
    if chart
        .routes()
        .iter()
        .any(|r| r.created_utc_millis > scanner.utc_millis)
    {
        return Err(invalid("chart route timestamp ahead of clock"));
    }
    Ok(chart)
}
impl GameplaySession {
    /// Trusted recovered-message/inference admission, consumed by #470. No
    /// coordinates are obtained from catalog lookup or inferred by this seam.
    #[allow(dead_code)] // Production source owner is downstream; exercised here.
    pub(crate) fn chart_receive_lead(
        &mut self,
        id: salimon_universe::CatalogId,
        lead: salimon_universe::scanner::chart::Lead,
    ) -> Result<(), SaveError> {
        if self.mining.terrain != salimon_world::terrain::TerrainMode::Canonical {
            return Err(invalid("chart lead requires canonical world"));
        }
        let mut candidate = self.chart.clone();
        candidate
            .record_lead(&CATALOG, id, lead)
            .map_err(|e| invalid(e.to_string()))?;
        self.chart = restore(
            Some(candidate.checkpoint()),
            salimon_persistence::SCHEMA_VERSION,
            true,
            &self.scanner,
        )?;
        Ok(())
    }
    pub(crate) fn chart_toggle(&mut self) {
        if self.scanner_controls_available() && self.ship.snapshot().power_available {
            self.scanner.chart_mode = !self.scanner.chart_mode;
            self.scanner.selected = 0;
            self.scanner.chart_note = None;
        }
    }
    pub(crate) fn chart_route_selected(&mut self) {
        if !self.scanner_controls_available()
            || !self.scanner.chart_mode
            || !self.ship.snapshot().power_available
        {
            return;
        }
        let count = self.chart.entries().count();
        let Some(index) = self.scanner.selected.checked_rem(count) else {
            return;
        };
        let id = self
            .chart
            .entries()
            .nth(index)
            .expect("bounded chart index")
            .id
            .clone();
        let from = Position::BodyLocal {
            body: salimon_world::canonical::START.manifest().planet.id.clone(),
            meters: self.ship.snapshot().pose.position_meters,
        };
        self.scanner.chart_note = Some(
            if self
                .chart
                .route_to(&id, from, self.scanner.utc_millis)
                .is_ok()
            {
                "ROUTE SAVED - NO AUTOPILOT"
            } else {
                "ROUTE NEEDS OBSERVED SAME-FRAME COORDINATES"
            },
        );
    }
    pub(crate) fn chart_hint(&self) -> String {
        let entries: Vec<_> = self.chart.entries().collect();
        let Some(index) = self.scanner.selected.checked_rem(entries.len()) else {
            return "CHART EMPTY - EXPLORE TO DISCOVER | P SCANNER".into();
        };
        let entry = entries[index];
        let name = if entry.observed().is_some()
            && entry.id == salimon_world::canonical::START.manifest().planet.id
        {
            salimon_world::canonical::START
                .manifest()
                .planet
                .display_name
                .as_str()
        } else if entry.observed().is_some() {
            CATALOG
                .get(&entry.id)
                .and_then(|r| r.header().display_name.as_deref())
                .unwrap_or("OBSERVED OBJECT")
        } else {
            "UNVERIFIED LEAD"
        };
        let provenance = match entry.observed() {
            Some(e) if e.fresh(self.scanner.utc_millis) => "OBSERVED",
            Some(_) => "HISTORICAL",
            None => {
                if entry.leads().last().is_some_and(|l| l.inferred) {
                    "INFERRED"
                } else {
                    "TRANSMITTED"
                }
            }
        };
        let uncertainty = entry
            .leads()
            .last()
            .filter(|_| entry.observed().is_none())
            .map(|l| match l.uncertainty {
                salimon_universe::scanner::chart::CoordinateUncertainty::Unknown => {
                    " UNCERTAINTY UNKNOWN".to_owned()
                }
                salimon_universe::scanner::chart::CoordinateUncertainty::RadiusMeters {
                    meters,
                } => format!(" +/-{meters:.0}m"),
            })
            .unwrap_or_default();
        let power = if self.scanner.powered {
            ""
        } else {
            " | PROJECTION OFF"
        };
        let note = self
            .scanner
            .chart_note
            .unwrap_or("M NEXT | ENTER ROUTE | P SCANNER");
        let ship = self.ship.snapshot();
        let origin = Position::BodyLocal {
            body: salimon_world::canonical::START.manifest().planet.id.clone(),
            meters: ship.pose.position_meters,
        };
        let coordinates = entry
            .position()
            .and_then(|position| position.relative_meters(&origin).ok())
            .map_or_else(
                || "FRAME UNAVAILABLE".to_owned(),
                |v| {
                    let local = ship.pose.axes().map(|axis| salimon_math::dot(v, axis));
                    let azimuth = if local[0].hypot(local[2]) < 1e-9 {
                        "AZ--".to_owned()
                    } else {
                        format!("AZ{:.0}", local[2].atan2(local[0]).to_degrees())
                    };
                    format!(
                        "SAMPLE {:.1}km {} EL{:.0}",
                        v[0].hypot(v[1]).hypot(v[2]) / 1000.,
                        azimuth,
                        local[1].atan2(local[0].hypot(local[2])).to_degrees()
                    )
                },
            );
        format!(
            "CHART {}/{} {} {}{} | {} | ROUTES {} | {}{}",
            index + 1,
            entries.len(),
            name,
            provenance,
            uncertainty,
            coordinates,
            self.chart.routes().len(),
            note,
            power
        )
    }
    pub(crate) fn chart_geometry(
        &self,
        pose: salimon_ship::ShipPose,
    ) -> Vec<salimon_renderer::SceneInstance> {
        if self.mining.terrain != salimon_world::terrain::TerrainMode::Canonical {
            return Vec::new();
        }
        let origin = Position::BodyLocal {
            body: salimon_world::canonical::START.manifest().planet.id.clone(),
            meters: self.ship.snapshot().pose.position_meters,
        };
        let axes = self.ship.snapshot().pose.axes();
        let entries: Vec<_> = self.chart.entries().collect();
        let selected = self.scanner.selected.checked_rem(entries.len());
        let mut range = self.scanner.parameters.range_meters;
        let contacts: Vec<_> = entries
            .iter()
            .enumerate()
            .filter_map(|(i, e)| {
                e.position()
                    .and_then(|position| position.relative_meters(&origin).ok())
                    .map(|v| {
                        range = range.max(v[0].hypot(v[1]).hypot(v[2]));
                        salimon_renderer::HologramContact {
                            relative_ship_meters: axes.map(|axis| salimon_math::dot(v, axis)),
                            selected: Some(i) == selected,
                            state: match e.observed() {
                                Some(known) if known.fresh(self.scanner.utc_millis) => {
                                    salimon_renderer::HologramContactState::Fresh
                                }
                                Some(_) => salimon_renderer::HologramContactState::Historical,
                                None => salimon_renderer::HologramContactState::Lead,
                            },
                        }
                    })
            })
            .collect();
        let frame = super::frames::character_ship_frame(pose);
        salimon_renderer::scanner_hologram(&contacts, self.scanner.powered, range)
            .into_iter()
            .map(|mut i| {
                i.center_meters = frame.local_to_world(i.center_meters);
                i.orientation = pose.orientation.map(|v| v as f32);
                i
            })
            .collect()
    }
}
#[cfg(test)]
mod tests;
