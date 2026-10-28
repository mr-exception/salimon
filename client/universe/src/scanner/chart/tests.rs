use super::*;
#[test]
fn chart_unknown_earth_no_catalog_reveal_lead_provenance_and_verified_route() {
    let (base, o) = super::super::tests::fixture();
    // Synthetic visibility fixture: Earth exists in catalog truth but has no
    // player observation, coordinates or route. This is not real source data.
    let earth = CatalogId::from_source("jpl", "399").unwrap();
    let mut records = base.records().to_vec();
    records.push(Record::Body(crate::Body {
        header: crate::Header {
            id: earth.clone(),
            display_name: Some("Earth".into()),
            provenance: crate::Provenance::Observed {
                dataset: base.datasets()[0].id.clone(),
                record_key: "synthetic-earth-visibility".into(),
            },
        },
        system: base.get(&o.id).unwrap().system().clone(),
        parent: o.id.clone(),
        body_kind: crate::BodyKind::Planet,
        mass_kg: None,
        radius_meters: None,
        definition: None,
    }));
    let c = Catalog::new(base.revision().into(), base.datasets().to_vec(), records).unwrap();
    assert!(c.get(&earth).is_some());
    let mut chart = Chart::new(c.revision()).unwrap();
    assert_eq!(chart.entries().count(), 0);
    assert!(chart.route_to(&o.id, o.position.clone(), 100).is_err());
    assert!(chart.get(&earth).is_none());
    chart
        .record_lead(
            &c,
            o.id.clone(),
            Lead {
                position: o.position.clone(),
                source: "recovered warning A".into(),
                received_utc_millis: 100,
                inferred: false,
                uncertainty: CoordinateUncertainty::Unknown,
            },
        )
        .unwrap();
    assert!(chart.get(&o.id).unwrap().observed().is_none());
    assert!(chart.route_to(&o.id, o.position.clone(), 100).is_err());
    let mut contacts = Contacts::new(c.revision()).unwrap();
    contacts
        .refresh(&c, 200, true, std::slice::from_ref(&o))
        .unwrap();
    chart.observe(&contacts).unwrap();
    chart.route_to(&o.id, o.position.clone(), 200).unwrap();
    assert_eq!(chart.routes().len(), 1);
    contacts.advance(200 + CONTACT_TTL_MILLIS).unwrap();
    chart.observe(&contacts).unwrap();
    assert_eq!(chart.entries().count(), 1);
    assert_eq!(chart.get(&o.id).unwrap().leads().len(), 1);
    assert_eq!(
        chart.get(&o.id).unwrap().leads()[0].uncertainty,
        CoordinateUncertainty::Unknown
    );
    let wire = serde_json::to_vec(&chart.checkpoint()).unwrap();
    let loaded = Chart::from_checkpoint(&c, serde_json::from_slice(&wire).unwrap()).unwrap();
    assert_eq!(loaded, chart);
    assert!(loaded.get(&earth).is_none());
    contacts
        .refresh(&c, 300 + CONTACT_TTL_MILLIS, true, std::slice::from_ref(&o))
        .unwrap();
    chart.observe(&contacts).unwrap();
    assert_eq!(
        chart
            .get(&o.id)
            .unwrap()
            .observed()
            .unwrap()
            .successful_refreshes(),
        2
    );
    assert_eq!(chart.routes().len(), 1);
    let mut earth_observation = o.clone();
    earth_observation.id = earth.clone();
    contacts
        .refresh(&c, 400 + CONTACT_TTL_MILLIS, true, &[earth_observation])
        .unwrap();
    chart.observe(&contacts).unwrap();
    assert!(chart.get(&earth).unwrap().observed().is_some());
    let mut bad = chart.checkpoint();
    bad.entries.push(bad.entries[0].clone());
    assert!(Chart::from_checkpoint(&c, bad).is_err());
}
#[test]
fn chart_inference_keeps_uncertainty_and_rejects_unproven_history() {
    let (c, o) = super::super::tests::fixture();
    let mut chart = Chart::new(c.revision()).unwrap();
    let mut lead = Lead {
        position: o.position.clone(),
        source: "inferred bearing".into(),
        received_utc_millis: 1,
        inferred: true,
        uncertainty: CoordinateUncertainty::RadiusMeters { meters: 500. },
    };
    chart.record_lead(&c, o.id.clone(), lead.clone()).unwrap();
    assert!(chart.get(&o.id).unwrap().leads()[0].inferred);
    let before = chart.clone();
    lead.uncertainty = CoordinateUncertainty::RadiusMeters { meters: f64::NAN };
    assert!(chart.record_lead(&c, o.id.clone(), lead).is_err());
    assert_eq!(chart, before);
    let mut bad = chart.checkpoint();
    bad.entries[0].leads.clear();
    assert!(Chart::from_checkpoint(&c, bad).is_err());
    let mut bad = chart.checkpoint();
    bad.catalog_revision = "wrong".into();
    assert!(Chart::from_checkpoint(&c, bad).is_err());
}

#[test]
fn chart_source_replays_are_idempotent_and_old_observation_history_cannot_revert_state() {
    let (c, o) = super::super::tests::fixture();
    let mut chart = Chart::new(c.revision()).unwrap();
    let lead = Lead {
        position: o.position.clone(),
        source: "versioned-record-v1".into(),
        received_utc_millis: 10,
        inferred: true,
        uncertainty: CoordinateUncertainty::Unknown,
    };
    chart.record_lead(&c, o.id.clone(), lead.clone()).unwrap();
    chart.record_lead(&c, o.id.clone(), lead.clone()).unwrap();
    assert_eq!(chart.get(&o.id).unwrap().leads().len(), 1);
    let before = chart.clone();
    let mut changed = lead;
    changed.received_utc_millis += 1;
    assert!(chart.record_lead(&c, o.id.clone(), changed).is_err());
    assert_eq!(chart, before);
    let mut contacts = Contacts::new(c.revision()).unwrap();
    contacts
        .refresh(&c, 100, true, std::slice::from_ref(&o))
        .unwrap();
    let old = contacts.clone();
    contacts
        .refresh(&c, 200, true, std::slice::from_ref(&o))
        .unwrap();
    chart.observe(&contacts).unwrap();
    let before = chart.clone();
    assert!(chart.observe(&old).is_err());
    assert_eq!(chart, before);
}
