//! Personal surface annotations and deliberate inspections, never a POI catalog.
use crate::{
    Epoch, Error, invalid,
    surface_observations::{Book, Estimate, Mark},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Edit {
    pub epoch: Epoch,
    pub note: String,
    pub deleted: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Inspection {
    pub site_id: String,
    pub publication: String,
    pub kind: String,
    pub epoch: Epoch,
    pub location: Estimate,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Annotation {
    pub edits: Vec<Edit>,
    pub inspections: Vec<Inspection>,
}
impl Annotation {
    pub fn inspected(&self) -> Option<&Inspection> {
        self.inspections.last()
    }
    pub fn deleted(&self) -> bool {
        self.edits.last().is_some_and(|e| e.deleted)
    }
    pub fn note(&self) -> &str {
        self.edits.last().map_or("", |e| e.note.as_str())
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub mark_id: u64,
    pub epoch: Epoch,
    pub location: Estimate,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Chart {
    pub version: u32,
    #[serde(deserialize_with = "deserialize_entries")]
    pub entries: BTreeMap<u64, Annotation>,
    pub targets: Vec<Target>,
    pub target: Option<u64>,
}
pub struct InspectionSample {
    pub site_id: String,
    pub publication: String,
    pub kind: String,
    pub site: [f64; 3],
    pub observer: [f64; 3],
    pub epoch: Epoch,
}
impl Default for Chart {
    fn default() -> Self {
        Self {
            version: 1,
            entries: BTreeMap::new(),
            targets: vec![],
            target: None,
        }
    }
}
impl Chart {
    pub fn sync(&mut self, book: &Book) -> Result<(), Error> {
        let mut next = self.clone();
        for m in book.marks() {
            next.entries.entry(m.id).or_insert_with(|| Annotation {
                edits: vec![Edit {
                    epoch: m.epoch,
                    note: m.note.clone(),
                    deleted: false,
                }],
                inspections: vec![],
            });
        }
        next.validate(book)?;
        *self = next;
        Ok(())
    }
    pub fn validate(&self, book: &Book) -> Result<(), Error> {
        book.validate()?;
        if self.version != 1
            || self.entries.len() > 1024
            || self.targets.len() > 128
            || self.entries.values().map(|a| a.edits.len()).sum::<usize>() > 4096
        {
            return Err(invalid("surface chart version/capacity"));
        }
        for (id, a) in &self.entries {
            let m = mark(book, *id)?;
            let mut previous = m.epoch;
            let mut deleted = false;
            if a.edits.is_empty()
                || a.edits.len() > 32
                || a.inspections.len() > 32
                || a.edits
                    .first()
                    .is_none_or(|e| e.epoch != m.epoch || e.note != m.note || e.deleted)
            {
                return Err(invalid("surface chart edit history"));
            }
            for e in &a.edits {
                if deleted
                    || e.epoch.seconds_since(previous)? < 0.
                    || e.note.chars().count() > 256
                    || e.note.chars().any(char::is_control)
                {
                    return Err(invalid("surface chart note/time"));
                }
                previous = e.epoch;
                deleted = e.deleted;
            }
            for p in &a.inspections {
                if !hash(&p.site_id)
                    || !hash(&p.publication)
                    || !matches!(p.kind.as_str(), "geological" | "biological")
                    || p.epoch.seconds_since(m.epoch)? < 0.
                {
                    return Err(invalid("surface chart inspection provenance"));
                }
                validate_location(&p.location)?;
                if p.location.measured_at != p.epoch {
                    return Err(invalid("inspection measurement epoch"));
                }
            }
        }
        for t in &self.targets {
            let m = mark(book, t.mark_id)?;
            validate_location(&t.location)?;
            let a = self
                .entries
                .get(&t.mark_id)
                .ok_or_else(|| invalid("target annotation missing"))?;
            if !(m.bearing.is_none() && t.location == m.estimate)
                && !a.inspections.iter().any(|p| {
                    p.location == t.location
                        && t.epoch.seconds_since(p.epoch).is_ok_and(|s| s >= 0.)
                })
            {
                return Err(invalid("target lacks measured location receipt"));
            }

            if t.epoch.seconds_since(t.location.measured_at)? < 0.
                || t.epoch.seconds_since(m.epoch)? < 0.
            {
                return Err(invalid("surface target timestamp"));
            }
        }
        if self.target.is_some_and(|id| {
            self.entries.get(&id).is_none_or(Annotation::deleted)
                || !self.targets.iter().any(|t| t.mark_id == id)
        }) {
            return Err(invalid("surface target deleted or unknown"));
        }
        Ok(())
    }
    pub fn active_ids(&self) -> Vec<u64> {
        self.entries
            .iter()
            .filter(|(_, a)| !a.deleted())
            .map(|(id, _)| *id)
            .collect()
    }
    pub fn edit(
        &mut self,
        book: &Book,
        id: u64,
        note: String,
        deleted: bool,
        epoch: Epoch,
    ) -> Result<(), Error> {
        let mut next = self.clone();
        next.sync(book)?;
        let a = next
            .entries
            .get_mut(&id)
            .ok_or_else(|| invalid("unknown personal mark"))?;
        if a.deleted() {
            return Err(invalid("personal mark already deleted"));
        }
        a.edits.push(Edit {
            epoch,
            note,
            deleted,
        });
        if deleted && next.target == Some(id) {
            next.target = None;
        }
        next.validate(book)?;
        *self = next;
        Ok(())
    }
    pub fn route(&mut self, book: &Book, id: u64, epoch: Epoch) -> Result<(), Error> {
        let mut next = self.clone();
        next.sync(book)?;
        let m = mark(book, id)?;
        let a = next
            .entries
            .get(&id)
            .ok_or_else(|| invalid("unknown personal mark"))?;
        if a.deleted() {
            return Err(invalid("deleted personal mark"));
        }
        let location = if let Some(p) = a.inspected() {
            p.location.clone()
        } else {
            if m.bearing.is_some() {
                return Err(invalid("bearing has unknown range"));
            }
            m.estimate.clone()
        };
        validate_location(&location)?;
        next.targets.push(Target {
            mark_id: id,
            epoch,
            location,
        });
        next.target = Some(id);
        next.validate(book)?;
        *self = next;
        Ok(())
    }
    /// Source adapter supplies the actual nearby physical source and observer.
    /// The stored location is an uncertain optical offset from measured position.
    pub fn inspect(&mut self, book: &Book, id: u64, sample: InspectionSample) -> Result<(), Error> {
        let InspectionSample {
            site_id,
            publication,
            kind,
            site,
            observer,
            epoch,
        } = sample;
        let mut next = self.clone();
        next.sync(book)?;
        let m = mark(book, id)?;
        let estimate = &book.estimate;
        let Some(origin) = estimate.position_meters else {
            return Err(invalid("inspection navigation unavailable"));
        };
        let Some(radius) = estimate.radius_meters else {
            return Err(invalid("inspection error unavailable"));
        };
        if !book.enabled
            || !site.iter().chain(&observer).all(|x| x.is_finite())
            || norm(sub(site, observer)) > 10.
            || norm(sub(origin, observer)) > radius
        {
            return Err(invalid(
                "inspection requires close measured physical source",
            ));
        }
        let mark_origin = m
            .estimate
            .position_meters
            .ok_or_else(|| invalid("mark has no body-frame origin"))?;
        let v = sub(site, mark_origin);
        let distance = norm(v);
        if let Some(direction) = m.bearing {
            if distance > 0.
                && (dot(v, direction) / distance).clamp(-1., 1.).acos()
                    > m.bearing_error_radians.unwrap_or(0.)
                        + (m.estimate.radius_meters.unwrap_or(0.) / distance)
                            .min(1.)
                            .asin()
            {
                return Err(invalid("source outside observed bearing uncertainty"));
            }
        } else if distance > m.estimate.radius_meters.unwrap_or(0.) + 10. {
            return Err(invalid("source outside marked area"));
        }
        let offset = sub(site, observer).map(|x| (x * 2.).round() / 2.);
        let location = Estimate {
            position_meters: Some(std::array::from_fn(|i| origin[i] + offset[i])),
            radius_meters: Some(radius + 0.5),
            source: estimate.source,
            measured_at: epoch,
        };
        let a = next
            .entries
            .get_mut(&id)
            .ok_or_else(|| invalid("unknown mark"))?;
        if a.deleted() {
            return Err(invalid("deleted mark"));
        }
        a.inspections.push(Inspection {
            site_id,
            publication,
            kind,
            epoch,
            location,
        });
        next.validate(book)?;
        *self = next;
        Ok(())
    }
}
fn deserialize_entries<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<BTreeMap<u64, Annotation>, D::Error> {
    struct Entries;
    impl<'de> serde::de::Visitor<'de> for Entries {
        type Value = BTreeMap<u64, Annotation>;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("unique bounded personal annotations")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut map: A,
        ) -> Result<Self::Value, A::Error> {
            let mut entries = BTreeMap::new();
            while let Some((id, entry)) = map.next_entry()? {
                if entries.len() >= 1024 || entries.insert(id, entry).is_some() {
                    return Err(serde::de::Error::custom(
                        "duplicate/oversized personal annotation map",
                    ));
                }
            }
            Ok(entries)
        }
    }
    d.deserialize_map(Entries)
}
fn mark(book: &Book, id: u64) -> Result<&Mark, Error> {
    book.marks()
        .iter()
        .find(|m| m.id == id)
        .ok_or_else(|| invalid("chart references unknown observation"))
}
fn hash(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}
fn norm(v: [f64; 3]) -> f64 {
    v[0].hypot(v[1]).hypot(v[2])
}
fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    (0..3).map(|i| a[i] * b[i]).sum()
}
fn validate_location(e: &Estimate) -> Result<(), Error> {
    if e.position_meters
        .is_none_or(|p| p.iter().any(|x| !x.is_finite()))
        || e.radius_meters.is_none_or(|r| !r.is_finite() || r <= 0.)
        || e.source == crate::surface_observations::Source::Unavailable
    {
        return Err(invalid("surface chart target needs uncertain coordinates"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn epoch(s: i64) -> Epoch {
        crate::Epoch::calendar([2000, 1, 1], [12, 0, 0, 0], crate::TimeScale::Tt)
            .unwrap()
            .advance_nanos(s * 1_000_000_000)
            .unwrap()
    }
    fn book() -> Book {
        let mut b = Book::new(
            crate::CatalogId::from_source("test", "planet").unwrap(),
            "a".repeat(64),
            epoch(0),
        )
        .unwrap();
        b.fix([0.; 3], epoch(0)).unwrap();
        b.mark(None, "parking".into(), epoch(0)).unwrap();
        b.mark(Some([1., 0., 0.]), "ridge".into(), epoch(0))
            .unwrap();
        b
    }
    #[test]
    fn surface_chart_personal_edits_targets_tombstones_and_history_preserve_observations() {
        let b = book();
        let original = b.clone();
        let mut c = Chart::default();
        c.sync(&b).unwrap();
        c.edit(&b, 1, "valley parking".into(), false, epoch(1))
            .unwrap();
        c.route(&b, 1, epoch(2)).unwrap();
        assert_eq!(c.target, Some(1));
        assert_eq!(c.targets[0].location, b.marks()[0].estimate);
        c.edit(&b, 1, "valley parking".into(), true, epoch(3))
            .unwrap();
        assert_eq!(c.target, None);
        assert_eq!(c.active_ids(), vec![2]);
        assert_eq!(c.entries[&1].edits.len(), 3);
        let restored: Chart = serde_json::from_slice(&serde_json::to_vec(&c).unwrap()).unwrap();
        restored.validate(&b).unwrap();
        assert_eq!(c, restored);
        assert_eq!(b, original);
        let previous = c.clone();
        assert!(c.edit(&b, 1, "resurrect".into(), false, epoch(4)).is_err());
        assert_eq!(c, previous);
    }
    #[test]
    fn surface_chart_unknown_bearing_needs_close_matching_inspection_and_retains_uncertainty() {
        let mut b = book();
        let mut c = Chart::default();
        c.sync(&b).unwrap();
        let original = c.clone();
        assert!(c.route(&b, 2, epoch(0)).is_err());
        assert_eq!(c, original);
        // A real measured approach to the bearing's physical endpoint.
        b.fix([100.3, 0., 0.], epoch(10)).unwrap();
        let sample = |site| InspectionSample {
            site_id: "b".repeat(64),
            publication: "c".repeat(64),
            kind: "geological".into(),
            site,
            observer: [100.3, 0., 0.],
            epoch: epoch(10),
        };
        assert!(c.inspect(&b, 2, sample([100., 40., 0.])).is_err());
        assert_eq!(c, original);
        c.inspect(&b, 2, sample([102.2, 0., 0.])).unwrap();
        assert_eq!(c.entries[&2].inspections.len(), 1);
        let p = c.entries[&2].inspected().unwrap();
        assert_eq!(p.location.radius_meters, Some(5.5));
        assert_ne!(p.location.position_meters, Some([102.2, 0., 0.]));
        c.route(&b, 2, epoch(11)).unwrap();
        c.inspect(&b, 2, sample([102.2, 0., 0.])).unwrap();
        assert_eq!(c.entries[&2].inspections.len(), 2);
        c.validate(&b).unwrap();
        let mut forged = c.clone();
        forged.targets[0].location.position_meters = Some([9999.; 3]);
        assert!(forged.validate(&b).is_err());
    }
    #[test]
    fn surface_chart_malformed_notes_unknown_ids_backwards_time_and_fake_targets_are_atomic() {
        let b = book();
        let mut c = Chart::default();
        c.sync(&b).unwrap();
        let old = c.clone();
        for (id, note, t) in [
            (999, "no".into(), epoch(1)),
            (1, "bad\n".into(), epoch(1)),
            (1, "x".repeat(257), epoch(1)),
            (1, "past".into(), epoch(-1)),
        ] {
            assert!(c.edit(&b, id, note, false, t).is_err());
            assert_eq!(c, old);
        }
        c.target = Some(2);
        assert!(c.validate(&b).is_err());
        let entry = serde_json::to_string(&old.entries[&1]).unwrap();
        let duplicate = format!(
            r#"{{"version":1,"entries":{{"1":{entry},"1":{entry}}},"targets":[],"target":null}}"#
        );
        assert!(serde_json::from_str::<Chart>(&duplicate).is_err());
    }
}
