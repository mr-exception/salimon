//! Domain-owned checkpoint mapping and historical schema admission.
use crate::session::GameplaySession;
use salimon_persistence::{SCHEMA_VERSION, SaveError, Validate, invalid};

pub(in crate::session) fn restore(
    source: Option<Snapshot>,
    version: u32,
) -> Result<Snapshot, SaveError> {
    let mut survival = match (version, source) {
        (1, None) => Snapshot {
            vitals: Default::default(),
            catalog: salimon_survival::consumables::Catalog::starter(),
            restrictions: Default::default(),
            run: Default::default(),
        },
        (2..=SCHEMA_VERSION, Some(state)) => state,
        _ => return Err(invalid("survival payload does not match envelope version")),
    };
    if version == 2 {
        // Schema 2 recorded only a cumulative counter. There is no missing
        // movement to infer, and no restore displacement to charge.
        survival.run.journey.initialize_tracking();
    } else if survival.run.journey.tracking.is_none() {
        return Err(invalid("schema-3 journey sampling source missing"));
    }
    survival.validate()?;
    Ok(survival)
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::session) struct Snapshot {
    pub(in crate::session) vitals: salimon_survival::Survival,
    pub(in crate::session) catalog: salimon_survival::consumables::Catalog,
    pub(in crate::session) restrictions: salimon_survival::restrictions::RestrictionConfig,
    pub(in crate::session) run: crate::session::lifecycle::RunState,
}
impl Validate for Snapshot {
    fn validate(&self) -> Result<(), SaveError> {
        self.vitals.validate()?;
        self.catalog.validate_registered(&self.vitals)?;
        self.restrictions.validate()?;
        self.run.validate()
    }
}

impl Snapshot {
    pub(in crate::session) fn capture(s: &GameplaySession) -> Self {
        Self {
            vitals: s.survival.clone(),
            catalog: s.consumable_effects.clone(),
            restrictions: s.restriction_config,
            run: s.run.clone(),
        }
    }
}
