use super::*;
pub(super) fn fixture() -> (Catalog, Observation) {
    let c = Catalog::decode(include_bytes!("../../fixtures/schema-v1.json")).unwrap();
    let id = c
        .records()
        .iter()
        .find(|r| matches!(r, Record::Star(_)))
        .unwrap()
        .id()
        .clone();
    let o = Observation {
        id,
        position: Position::Galactic {
            position: GalacticPosition::new([5_000_000; 3], [0.125, 2., 3.]).unwrap(),
        },
        velocity_mps: [1., 2., 3.],
    };
    (c, o)
}
#[test]
fn scanner_unknown_power_expiry_history_and_rediscovery() {
    let (c, mut o) = fixture();
    let mut s = Contacts::new(c.revision()).unwrap();
    assert_eq!(s.history().count(), 0);
    s.refresh(&c, 1000, false, std::slice::from_ref(&o))
        .unwrap();
    assert_eq!(s.history().count(), 0);
    s.refresh(&c, 2000, true, std::slice::from_ref(&o)).unwrap();
    assert_eq!(s.active(2000 + CONTACT_TTL_MILLIS - 1).count(), 1);
    s.refresh(&c, 3000, false, std::slice::from_ref(&o))
        .unwrap();
    assert_eq!(s.active(2000 + CONTACT_TTL_MILLIS).count(), 0);
    assert_eq!(s.history().count(), 1);
    o.position = o.position.translated([2., 0., 0.]).unwrap();
    s.refresh(
        &c,
        2000 + CONTACT_TTL_MILLIS,
        true,
        std::slice::from_ref(&o),
    )
    .unwrap();
    let e = s.active(2000 + CONTACT_TTL_MILLIS).next().unwrap();
    assert_eq!(e.observation(), &o);
    assert_eq!(e.first_seen_utc_millis(), 2000);
    assert_eq!(e.successful_refreshes(), 2);
}
#[test]
fn scanner_rejects_atomic_invalid_identity_duplicate_time_and_nonfinite() {
    let (c, o) = fixture();
    let mut s = Contacts::new(c.revision()).unwrap();
    s.refresh(&c, 100, true, std::slice::from_ref(&o)).unwrap();
    let before = s.clone();
    let mut bad = o.clone();
    bad.id = CatalogId::from_source("missing", "star").unwrap();
    assert!(s.refresh(&c, 200, true, &[o.clone(), bad]).is_err());
    assert_eq!(s, before);
    assert!(s.refresh(&c, 200, true, &[o.clone(), o.clone()]).is_err());
    assert!(s.refresh(&c, 99, true, &[]).is_err());
    let mut bad = o;
    bad.velocity_mps[0] = f64::NAN;
    assert!(s.refresh(&c, 200, true, &[bad]).is_err());
    assert_eq!(s, before);
    let mut wrong = Contacts::new("other-revision").unwrap();
    assert!(wrong.refresh(&c, 200, true, &[]).is_err());
}
