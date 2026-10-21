use crate::*;
use std::collections::{BTreeMap, BTreeSet};
fn text(value: &str, max: usize, label: &str) -> Result<(), Error> {
    if value.is_empty()
        || value.len() > max
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err(invalid(format!("invalid {label}")));
    }
    Ok(())
}
fn digest(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn provenance(p: &Provenance, sources: &BTreeSet<&CatalogId>) -> Result<(), Error> {
    match p {
        Provenance::Observed {
            dataset,
            record_key,
        } => {
            if !sources.contains(dataset) {
                return Err(invalid(format!(
                    "missing observed dataset {}",
                    dataset.as_str()
                )));
            }
            text(record_key, 512, "source record key")?;
        }
        Provenance::Inferred { method, datasets } => {
            text(method, 1024, "inference method")?;
            let unique: BTreeSet<_> = datasets.iter().collect();
            if unique.len() != datasets.len()
                || datasets.is_empty()
                || datasets.iter().any(|id| !sources.contains(id))
            {
                return Err(invalid(
                    "inference requires unique registered input datasets",
                ));
            }
        }
        Provenance::Fiction {
            builder,
            revision,
            rationale,
        } => {
            text(builder, 256, "fiction builder")?;
            text(revision, 160, "builder revision")?;
            text(rationale, 2048, "fiction rationale")?;
        }
    }
    Ok(())
}
fn quantity(q: &Option<Quantity>, sources: &BTreeSet<&CatalogId>) -> Result<(), Error> {
    let Some(q) = q else { return Ok(()) };
    if !q.value.is_finite() || q.value <= 0.0 {
        return Err(invalid("physical SI quantity must be positive and finite"));
    }
    provenance(&q.provenance, sources)?;
    if let Uncertainty::Bounds {
        minus,
        plus,
        confidence,
    } = q.uncertainty
        && (!minus.is_finite()
            || !plus.is_finite()
            || minus < 0.0
            || plus < 0.0
            || minus > q.value
            || !(q.value + plus).is_finite()
            || confidence.is_some_and(|c| !c.is_finite() || c <= 0.0 || c > 1.0))
    {
        return Err(invalid("quantity uncertainty bounds/confidence"));
    }
    if let Some(v) = &q.validity
        && (!v.start_julian_day.is_finite()
            || !v.end_julian_day.is_finite()
            || v.start_julian_day > v.end_julian_day)
    {
        return Err(invalid("quantity validity interval"));
    }
    Ok(())
}
fn definition(d: &Definition) -> Result<(), Error> {
    if d.version == 0 || !digest(&d.sha256) || (d.domains.is_empty() || d.domains.len() > 64) {
        return Err(invalid("definition revision, SHA-256 or seed domains"));
    }
    let mut domains = BTreeMap::new();
    for domain in &d.domains {
        text(&domain.name, 64, "seed domain name")?;
        if !domain
            .name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            || domain.algorithm_version == 0
            || !digest(&domain.definition_sha256)
            || domains.insert(domain.name.as_str(), domain).is_some()
        {
            return Err(invalid("duplicate/invalid seed domain or digest"));
        }
    }
    for domain in &d.domains {
        let deps: BTreeSet<_> = domain.dependencies.iter().collect();
        if deps.len() != domain.dependencies.len()
            || domain
                .dependencies
                .iter()
                .any(|name| !domains.contains_key(name.as_str()))
        {
            return Err(invalid("missing/duplicate seed dependency"));
        }
    }
    // Small named-domain graph; iterative walk avoids recursion on untrusted input.
    let mut resolved = BTreeSet::new();
    loop {
        let before = resolved.len();
        for domain in &d.domains {
            if domain
                .dependencies
                .iter()
                .all(|name| resolved.contains(name.as_str()))
            {
                resolved.insert(domain.name.as_str());
            }
        }
        if resolved.len() == d.domains.len() {
            break;
        }
        if resolved.len() == before {
            return Err(invalid("cyclic seed-domain dependencies"));
        }
    }
    Ok(())
}
impl Catalog {
    pub(crate) fn validate(&self) -> Result<(), Error> {
        text(&self.revision, 160, "catalog revision")?;
        let mut sources = BTreeSet::new();
        for source in &self.datasets {
            if !sources.insert(&source.id) {
                return Err(invalid(format!("duplicate dataset {}", source.id.as_str())));
            }
            text(&source.version, 160, "dataset version")?;
            text(&source.reference, 4096, "dataset reference")?;
        }
        let mut records = BTreeMap::new();
        for record in &self.records {
            if records.insert(record.id(), record).is_some() || sources.contains(record.id()) {
                return Err(invalid(format!(
                    "duplicate canonical identity {}",
                    record.id().as_str()
                )));
            }
        }
        for record in &self.records {
            let result = (|| {
                let h = record.header();
                if let Some(name) = &h.display_name {
                    text(name, 256, "display name")?;
                }
                provenance(&h.provenance, &sources)?;
                if !matches!(records.get(record.system()), Some(Record::System(_))) {
                    return Err(invalid("missing/wrong-kind system reference"));
                }
                if let Some(parent) = record.parent() {
                    let p = records
                        .get(parent)
                        .ok_or_else(|| invalid("missing parent reference"))?;
                    if p.system() != record.system() || matches!(p, Record::Population(_)) {
                        return Err(invalid("parent has wrong system/kind"));
                    }
                }
                match record {
                    Record::System(_) => {}
                    Record::Star(r) => {
                        quantity(&r.mass_kg, &sources)?;
                        quantity(&r.radius_meters, &sources)?;
                    }
                    Record::Body(r) => {
                        quantity(&r.mass_kg, &sources)?;
                        quantity(&r.radius_meters, &sources)?;
                        if r.body_kind == BodyKind::Moon
                            && !matches!(records.get(&r.parent), Some(Record::Body(_)))
                        {
                            return Err(invalid("moon parent must be an individual body"));
                        }
                        if r.body_kind == BodyKind::Planet
                            && !matches!(
                                records.get(&r.parent),
                                Some(Record::Star(_) | Record::System(_))
                            )
                        {
                            return Err(invalid("planet parent must be star or system"));
                        }
                        if let Some(d) = &r.definition {
                            definition(d)?;
                        }
                        if matches!(r.header.provenance, Provenance::Fiction { .. })
                            && matches!(r.body_kind, BodyKind::Planet | BodyKind::Moon)
                        {
                            let d = r.definition.as_ref().ok_or_else(|| {
                                invalid("fictional world requires canonical definition identity")
                            })?;
                            for name in ["terrain", "resources", "visual"] {
                                if !d.domains.iter().any(|d| d.name == name) {
                                    return Err(invalid(format!(
                                        "fictional world missing {name} seed domain"
                                    )));
                                }
                            }
                        }
                    }
                    Record::Population(r) => {
                        text(&r.descriptor, 2048, "statistical population descriptor")?;
                        definition(&r.definition)?;
                    }
                }
                Ok(())
            })();
            result.map_err(|e: Error| invalid(format!("record {}: {e}", record.id().as_str())))?;
        }
        // Each parent chain is resolved once; cycles cannot publish a catalog.
        let mut resolved = BTreeSet::new();
        for record in &self.records {
            let mut path = BTreeSet::new();
            let mut current = Some(record.id());
            while let Some(id) = current {
                if resolved.contains(id) {
                    break;
                }
                if !path.insert(id) {
                    return Err(invalid(format!("parent cycle at {}", id.as_str())));
                }
                current = records[id].parent();
            }
            resolved.extend(path);
        }
        Ok(())
    }
}
