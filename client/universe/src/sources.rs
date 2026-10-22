//! U06 source metadata only; offline tooling verifies raw artifact content hashes.
use crate::{CatalogId, Error, invalid};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceManifest {
    pub format: String,
    pub schema_version: u32,
    pub reviewed_on: String,
    pub sources: Vec<Source>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub id: CatalogId,
    pub release: String,
    pub endpoint: String,
    pub query: String,
    pub documentation: Vec<String>,
    pub license: License,
    pub coverage: String,
    pub uncertainty: String,
    pub artifacts: Vec<Artifact>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct License {
    pub status: ReuseStatus,
    pub policy_url: String,
    pub attribution: String,
    pub review_issue: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReuseStatus {
    Restricted,
    Unreviewed,
    Approved,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
    pub origin: Origin,
    pub encoding: Encoding,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    Synthetic,
    ObservedSnapshot,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Encoding {
    Csv,
    Json,
}
impl SourceManifest {
    pub fn decode(bytes: &[u8], production: bool) -> Result<Self, Error> {
        if bytes.len() > 1024 * 1024 {
            return Err(invalid("source manifest exceeds 1 MiB"));
        }
        let m: Self = serde_json::from_slice(bytes).map_err(|e| invalid(e.to_string()))?;
        m.validate(production)?;
        Ok(m)
    }
    pub fn validate(&self, production: bool) -> Result<(), Error> {
        if self.schema_version != 1 {
            return Err(Error::UnsupportedSchema(self.schema_version));
        }
        if self.format != "salimon.universe.sources"
            || self.reviewed_on.len() != 10
            || self.sources.is_empty()
        {
            return Err(invalid("source manifest header/empty sources"));
        }
        let mut ids = BTreeSet::new();
        let mut paths = BTreeSet::new();
        for s in &self.sources {
            if !ids.insert(&s.id)
                || [
                    &s.release,
                    &s.query,
                    &s.coverage,
                    &s.uncertainty,
                    &s.license.attribution,
                ]
                .into_iter()
                .any(|v| v.trim().is_empty())
                || !s.endpoint.starts_with("https://")
                || !s.license.policy_url.starts_with("https://")
                || !s.license.review_issue.starts_with("https://")
                || s.documentation.is_empty()
                || s.documentation.iter().any(|v| !v.starts_with("https://"))
                || s.artifacts.is_empty()
            {
                return Err(invalid("source identity, metadata or HTTPS reference"));
            }
            if production && s.license.status != ReuseStatus::Approved {
                return Err(invalid(format!(
                    "source {} reuse not approved",
                    s.id.as_str()
                )));
            }
            for a in &s.artifacts {
                if a.path.is_empty()
                    || a.path.starts_with('/')
                    || a.path.contains('\\')
                    || a.path
                        .split('/')
                        .any(|p| p.is_empty() || p == ".." || p == "." || p.contains(':'))
                    || !paths.insert(&a.path)
                    || a.bytes == 0
                    || a.sha256.len() != 64
                    || !a
                        .sha256
                        .bytes()
                        .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
                {
                    return Err(invalid("artifact path/checksum/size"));
                }
                if production && a.origin == Origin::Synthetic {
                    return Err(invalid(
                        "synthetic artifact cannot enter production catalog",
                    ));
                }
            }
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    const FIXTURE: &[u8] = include_bytes!("../../../catalogs/sources/manifest-v1.json");
    #[test]
    fn pinned_sources_decode_but_production_reuse_is_not_implicitly_approved() {
        let m = SourceManifest::decode(FIXTURE, false).unwrap();
        assert_eq!(m.sources.len(), 3);
        assert!(m.validate(true).is_err());
    }
    #[test]
    fn malformed_artifacts_and_unknown_versions_cannot_publish() {
        let m = SourceManifest::decode(FIXTURE, false).unwrap();
        for path in ["../secret", "/tmp/data", "C:/data", "a/../b", "a\\b"] {
            let mut bad = m.clone();
            bad.sources[0].artifacts[0].path = path.into();
            assert!(bad.validate(false).is_err());
        }
        let mut bad = m.clone();
        bad.schema_version = 2;
        assert!(matches!(
            bad.validate(false),
            Err(Error::UnsupportedSchema(2))
        ));
        let mut bad = m;
        bad.sources.push(bad.sources[0].clone());
        assert!(bad.validate(false).is_err());
    }
}
