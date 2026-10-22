use super::*;
use crate as universe;
#[path = "../../examples/support/reference_inputs.rs"]
mod reference_inputs;
fn suite() -> Suite {
    Suite::decode(include_bytes!("../../fixtures/references-v1.json")).unwrap()
}
fn actual() -> Vec<Sample> {
    reference_inputs::samples().unwrap()
}
#[test]
fn golden_jpl_epochs_si_conversion_and_production_frames_match() {
    let s = suite();
    let a = actual();
    let result = s.compare(&a).unwrap();
    assert!(result.passed, "{result:?}");
    assert_eq!(result.cases.len(), 5);
    assert_eq!(a[1].epoch.seconds_since(a[0].epoch).unwrap(), 86400.0);
    assert_eq!(a[0].epoch.scale(), crate::TimeScale::Tdb);
    assert!(
        result
            .cases
            .iter()
            .any(|c| matches!(c.regime, Regime::Interstellar))
    );
}
#[test]
fn independent_faults_cannot_silently_pass() {
    for fault in [
        "km",
        "axis",
        "epoch",
        "scale",
        "missing",
        "extra",
        "duplicate",
        "nan",
        "velocity",
        "frame",
        "id",
    ] {
        let mut a = actual();
        match fault {
            "km" => a[0].position_meters[0] /= 1000.,
            "axis" => a[0].position_meters.swap(0, 1),
            "epoch" => a[0].epoch = a[0].epoch.advance_nanos(1).unwrap(),
            "scale" => {
                a[0].epoch =
                    Epoch::calendar([2000, 1, 1], [0, 0, 0, 0], crate::TimeScale::Tt).unwrap()
            }
            "missing" => {
                a.pop();
            }
            "extra" => a.push(a[0].clone()),
            "duplicate" => a[1] = a[0].clone(),
            "nan" => a[0].position_meters[1] = f64::NAN,
            "velocity" => a[0].velocity_mps[1] += 0.01,
            "frame" => a[0].frame = "heliocentric-ecliptic".into(),
            _ => a[0].id = "unknown".into(),
        }
        assert!(!suite().compare(&a).is_ok_and(|r| r.passed), "{fault}");
    }
}
#[test]
fn norm_not_componentwise_and_boundary_inclusive() {
    let epoch = Epoch::calendar([2000, 1, 1], [0, 0, 0, 0], crate::TimeScale::Tdb).unwrap();
    let expected = Sample {
        id: "test".into(),
        frame: "test".into(),
        epoch,
        position_meters: [0.; 3],
        velocity_mps: [0.; 3],
    };
    let s = Suite::new(vec![Case {
        expected: expected.clone(),
        regime: Regime::LocalContact,
        budget: Budget {
            position_meters: 5.,
            velocity_mps: 0.,
        },
        budget_basis: "3-4-5 exact norm".into(),
        reference: "analytic".into(),
    }])
    .unwrap();
    let mut a = expected;
    a.position_meters = [3., 4., 0.];
    assert!(s.compare(&[a.clone()]).unwrap().passed);
    a.position_meters = [4., 4., 0.];
    assert!(!s.compare(&[a]).unwrap().passed);
}
#[test]
fn version_fields_bounds_duplicates_and_invalid_budgets_reject() {
    let bytes = include_bytes!("../../fixtures/references-v1.json");
    let original: serde_json::Value = serde_json::from_slice(bytes).unwrap();
    for fault in [
        "version",
        "unknown",
        "negative",
        "duplicate",
        "empty",
        "reference",
    ] {
        let mut v = original.clone();
        match fault {
            "version" => v["schema_version"] = 2.into(),
            "unknown" => v["cases"][0]["unexpected"] = true.into(),
            "negative" => v["cases"][0]["budget"]["position_meters"] = (-1).into(),
            "duplicate" => {
                let c = v["cases"][0].clone();
                v["cases"].as_array_mut().unwrap().push(c);
            }
            "empty" => v["cases"] = serde_json::json!([]),
            _ => v["cases"][0]["reference"] = "".into(),
        }
        assert!(
            Suite::decode(&serde_json::to_vec(&v).unwrap()).is_err(),
            "{fault}"
        );
    }
    assert!(Suite::decode(&vec![b' '; MAX_REFERENCE_BYTES + 1]).is_err());
    let mut cases = suite().cases().to_vec();
    cases[0].budget.position_meters = f64::INFINITY;
    assert!(Suite::new(cases).is_err());
    let duplicate=b"{\"format\":\"salimon.universe.references\",\"schema_version\":1,\"schema_version\":1,\"cases\":[]}";
    assert!(Suite::decode(duplicate).is_err());
}
#[test]
fn candidate_envelope_strict_roundtrip_order_and_overflow() {
    let mut a = actual();
    a.reverse();
    let bytes = serde_json::to_vec(
        &serde_json::json!({"format":"salimon.universe.samples","schema_version":1,"samples":a}),
    )
    .unwrap();
    assert!(
        suite()
            .compare(&decode_samples(&bytes).unwrap())
            .unwrap()
            .passed
    );
    assert!(decode_samples(b"[]").is_err());
    let mut v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    v["schema_version"] = 9.into();
    assert!(decode_samples(&serde_json::to_vec(&v).unwrap()).is_err());
    let mut c = suite().cases().to_vec();
    c[0].expected.position_meters = [-f64::MAX; 3];
    let s = Suite::new(c).unwrap();
    let mut a = s
        .cases()
        .iter()
        .map(|c| c.expected.clone())
        .collect::<Vec<_>>();
    a[0].position_meters = [f64::MAX; 3];
    assert!(s.compare(&a).is_err());
}
