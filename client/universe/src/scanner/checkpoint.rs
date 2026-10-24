//! Bounded contact checkpoint; reconstruction validates against the actual catalog.
use super::*;
pub const CHECKPOINT_VERSION: u32 = 1;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Checkpoint {
    pub version: u32,
    pub catalog_revision: String,
    pub watermark_utc_millis: u64,
    pub encounters: Vec<Encounter>,
}
impl Contacts {
    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint {
            version: CHECKPOINT_VERSION,
            catalog_revision: self.catalog_revision.clone(),
            watermark_utc_millis: self.watermark_utc_millis,
            encounters: self.history().cloned().collect(),
        }
    }
    pub fn from_checkpoint(catalog: &Catalog, source: Checkpoint) -> Result<Self, Error> {
        if source.version != CHECKPOINT_VERSION
            || source.catalog_revision != catalog.revision()
            || source.encounters.len() > MAX_CONTACTS
        {
            return Err(invalid("scanner checkpoint version/catalog/size"));
        }
        let mut state = Self::new(catalog.revision())?;
        state.watermark_utc_millis = source.watermark_utc_millis;
        for e in source.encounters {
            validate_observation(catalog, &e.observation)?;
            if e.first_seen_utc_millis > e.last_seen_utc_millis
                || e.last_seen_utc_millis > source.watermark_utc_millis
                || e.successful_refreshes == 0
                || state
                    .encounters
                    .insert(e.observation.id.clone(), e)
                    .is_some()
            {
                return Err(invalid("scanner checkpoint timestamps/count/duplicate"));
            }
        }
        Ok(state)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scanner_checkpoint_roundtrip_expiry_rescan_and_atomic_corruption() {
        let (c, o) = super::super::tests::fixture();
        let mut s = Contacts::new(c.revision()).unwrap();
        s.refresh(&c, 5000, true, std::slice::from_ref(&o)).unwrap();
        let bytes = serde_json::to_vec(&s.checkpoint()).unwrap();
        let decoded: Checkpoint = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(Contacts::from_checkpoint(&c, decoded).unwrap(), s);
        let mut loaded = Contacts::from_checkpoint(&c, s.checkpoint()).unwrap();
        loaded.advance(5000 + CONTACT_TTL_MILLIS).unwrap();
        assert_eq!(loaded.active(loaded.watermark_utc_millis).count(), 0);
        assert_eq!(loaded.history().count(), 1);
        loaded
            .refresh(
                &c,
                5000 + CONTACT_TTL_MILLIS,
                true,
                std::slice::from_ref(&o),
            )
            .unwrap();
        assert_eq!(loaded.history().next().unwrap().successful_refreshes(), 2);
        let mut bad = s.checkpoint();
        bad.encounters.push(bad.encounters[0].clone());
        assert!(Contacts::from_checkpoint(&c, bad).is_err());
        let mut bad = s.checkpoint();
        bad.catalog_revision = "different".into();
        assert!(Contacts::from_checkpoint(&c, bad).is_err());
        let mut bad = s.checkpoint();
        bad.encounters[0].last_seen_utc_millis += 1;
        assert!(Contacts::from_checkpoint(&c, bad).is_err());
        let mut bad = s.checkpoint();
        bad.encounters[0].successful_refreshes = 0;
        assert!(Contacts::from_checkpoint(&c, bad).is_err());
    }
}
