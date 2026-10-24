//! Player-known coordinates and voluntary route history, separate from catalog truth.
use super::*;
const MAX_ROUTES: usize = 4096;
const MAX_LEADS: usize = 256;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CoordinateUncertainty {
    Unknown,
    RadiusMeters { meters: f64 },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lead {
    pub position: Position,
    pub source: String,
    pub received_utc_millis: u64,
    pub inferred: bool,
    pub uncertainty: CoordinateUncertainty,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub id: CatalogId,
    observed: Option<Encounter>,
    leads: Vec<Lead>,
}
impl Entry {
    pub fn observed(&self) -> Option<&Encounter> {
        self.observed.as_ref()
    }
    pub fn leads(&self) -> &[Lead] {
        &self.leads
    }
    pub fn position(&self) -> Option<&Position> {
        self.observed
            .as_ref()
            .map(|e| &e.observation().position)
            .or_else(|| self.leads.last().map(|lead| &lead.position))
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Route {
    pub destination: CatalogId,
    pub from: Position,
    pub to: Position,
    pub created_utc_millis: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Chart {
    revision: String,
    entries: BTreeMap<CatalogId, Entry>,
    routes: Vec<Route>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChartCheckpoint {
    pub version: u32,
    pub catalog_revision: String,
    pub entries: Vec<Entry>,
    pub routes: Vec<Route>,
}
impl Chart {
    pub fn new(catalog_revision: &str) -> Result<Self, Error> {
        Contacts::new(catalog_revision)?;
        Ok(Self {
            revision: catalog_revision.into(),
            entries: BTreeMap::new(),
            routes: Vec::new(),
        })
    }
    pub fn entries(&self) -> impl Iterator<Item = &Entry> {
        self.entries.values()
    }
    pub fn get(&self, id: &CatalogId) -> Option<&Entry> {
        self.entries.get(id)
    }
    pub fn routes(&self) -> &[Route] {
        &self.routes
    }
    /// Only actual scanner encounter history grants verified celestial knowledge.
    /// Freshness and durable chart knowledge deliberately have different lifetimes.
    pub fn observe(&mut self, contacts: &Contacts) -> Result<(), Error> {
        if self.revision != contacts.catalog_revision() {
            return Err(invalid("chart/scanner catalog mismatch"));
        }
        let new = contacts
            .history()
            .filter(|e| !self.entries.contains_key(&e.observation().id))
            .count();
        if self.entries.len() + new > MAX_CONTACTS + MAX_LEADS {
            return Err(invalid("chart capacity"));
        }
        for e in contacts.history() {
            if self
                .get(&e.observation().id)
                .and_then(Entry::observed)
                .is_some_and(|prior| {
                    e.last_seen_utc_millis() < prior.last_seen_utc_millis()
                        || e.first_seen_utc_millis() != prior.first_seen_utc_millis()
                        || e.successful_refreshes() < prior.successful_refreshes()
                })
            {
                return Err(invalid("chart observation history regressed"));
            }
        }
        for e in contacts.history() {
            let id = e.observation().id.clone();
            self.entries
                .entry(id.clone())
                .or_insert_with(|| Entry {
                    id,
                    observed: None,
                    leads: Vec::new(),
                })
                .observed = Some(e.clone());
        }
        Ok(())
    }
    /// Coordinates are supplied by the trusted recovered-message/inference owner,
    /// never filled from a catalog search. A lead cannot verify itself or route.
    pub fn record_lead(
        &mut self,
        catalog: &Catalog,
        id: CatalogId,
        lead: Lead,
    ) -> Result<(), Error> {
        if self.revision != catalog.revision() {
            return Err(invalid("chart catalog mismatch"));
        }
        validate_lead(catalog, &id, &lead)?;
        if let Some(prior) = self
            .get(&id)
            .and_then(|e| e.leads.iter().find(|l| l.source == lead.source))
        {
            return if prior == &lead {
                Ok(())
            } else {
                Err(invalid("changed chart lead requires a new source identity"))
            };
        }
        if self.entries.values().map(|e| e.leads.len()).sum::<usize>() >= MAX_LEADS
            || (!self.entries.contains_key(&id) && self.entries.len() >= MAX_CONTACTS + MAX_LEADS)
        {
            return Err(invalid("chart lead capacity"));
        }
        self.entries
            .entry(id.clone())
            .or_insert_with(|| Entry {
                id,
                observed: None,
                leads: Vec::new(),
            })
            .leads
            .push(lead);
        Ok(())
    }
    /// Voluntary navigation annotation only. No autopilot, teleport or authority.
    /// Unknown/unverified targets and incompatible coordinate frames cannot route.
    pub fn route_to(&mut self, id: &CatalogId, from: Position, now: u64) -> Result<(), Error> {
        let entry = self
            .entries
            .get(id)
            .ok_or_else(|| invalid("destination undiscovered"))?;
        let observed = entry
            .observed
            .as_ref()
            .ok_or_else(|| invalid("destination requires verified discovery"))?;
        if now < observed.last_seen_utc_millis() || self.routes.len() >= MAX_ROUTES {
            return Err(invalid("chart route clock/capacity"));
        }
        let position = entry
            .position()
            .ok_or_else(|| invalid("chart entry has no coordinates"))?;
        position.relative_meters(&from)?;
        self.routes.push(Route {
            destination: id.clone(),
            from,
            to: position.clone(),
            created_utc_millis: now,
        });
        Ok(())
    }
    pub fn checkpoint(&self) -> ChartCheckpoint {
        ChartCheckpoint {
            version: 1,
            catalog_revision: self.revision.clone(),
            entries: self.entries.values().cloned().collect(),
            routes: self.routes.clone(),
        }
    }
    pub fn from_checkpoint(catalog: &Catalog, source: ChartCheckpoint) -> Result<Self, Error> {
        if source.version != 1
            || source.catalog_revision != catalog.revision()
            || source.entries.len() > MAX_CONTACTS + MAX_LEADS
            || source.routes.len() > MAX_ROUTES
        {
            return Err(invalid("chart checkpoint version/catalog/size"));
        }
        let mut chart = Self::new(catalog.revision())?;
        let mut lead_count = 0;
        for entry in source.entries {
            if entry.observed.is_none() && entry.leads.is_empty() {
                return Err(invalid("chart entry has no player evidence"));
            }
            if let Some(e) = &entry.observed {
                if entry.id != e.observation().id {
                    return Err(invalid("chart identity mismatch"));
                }
                Contacts::from_checkpoint(
                    catalog,
                    super::checkpoint::Checkpoint {
                        version: 1,
                        catalog_revision: chart.revision.clone(),
                        watermark_utc_millis: e.last_seen_utc_millis(),
                        encounters: vec![e.clone()],
                    },
                )?;
            }
            for lead in &entry.leads {
                validate_lead(catalog, &entry.id, lead)?;
                lead_count += 1;
            }
            if lead_count > MAX_LEADS || chart.entries.insert(entry.id.clone(), entry).is_some() {
                return Err(invalid("chart duplicate/lead capacity"));
            }
        }
        for route in source.routes {
            let entry = chart
                .get(&route.destination)
                .ok_or_else(|| invalid("route destination unknown"))?;
            if entry.observed.is_none() {
                return Err(invalid("unverified route"));
            }
            validate_position(catalog, &route.destination, &route.from)?;
            validate_position(catalog, &route.destination, &route.to)?;
            route.to.relative_meters(&route.from)?;
            if route.created_utc_millis
                < entry
                    .observed
                    .as_ref()
                    .expect("checked observed route")
                    .first_seen_utc_millis()
            {
                return Err(invalid("route predates discovery"));
            }
            chart.routes.push(route);
        }
        Ok(chart)
    }
}
fn validate_position(catalog: &Catalog, id: &CatalogId, position: &Position) -> Result<(), Error> {
    super::validate_position(catalog, position)?;
    if !matches!(catalog.get(id), Some(Record::Star(_) | Record::Body(_))) {
        return Err(invalid("chart requires celestial identity"));
    }
    Ok(())
}
fn validate_lead(catalog: &Catalog, id: &CatalogId, lead: &Lead) -> Result<(), Error> {
    validate_position(catalog, id, &lead.position)?;
    if lead.source.trim().is_empty()
        || lead.source.len() > 256
        || matches!(lead.uncertainty,CoordinateUncertainty::RadiusMeters{meters} if !meters.is_finite()||meters<0.)
    {
        return Err(invalid("chart lead provenance/uncertainty"));
    }
    Ok(())
}
#[cfg(test)]
mod tests;
