use salimon_persistence::{SaveError, invalid};

/// v1-v4 force capacities are deliberately superseded, not converted into an
/// arbitrary coefficient. Preserve energy, velocity and all other source state.
pub(super) fn migrate_propulsion(bytes: &[u8]) -> Result<Vec<u8>, SaveError> {
    if bytes.len() > salimon_persistence::MAX_SAVE_BYTES {
        return Err(invalid("save exceeds size limit"));
    }
    #[derive(serde::Deserialize)]
    struct Header {
        schema_version: u32,
    }
    let version = serde_json::from_slice::<Header>(bytes)
        .map_err(|e| SaveError::Encoding(e.to_string()))?
        .schema_version;
    if !(1..=salimon_persistence::SCHEMA_VERSION).contains(&version) {
        return Err(SaveError::UnsupportedVersion(version));
    }
    if version == salimon_persistence::SCHEMA_VERSION {
        // Preserve typed decoding's duplicate-field rejection, and avoid a
        // full DOM/reserialization pass for ordinary current-schema restores.
        return Ok(bytes.to_vec());
    }
    let mut value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| SaveError::Encoding(e.to_string()))?;
    if version <= 4 {
        let ship = value["state"]["physical"]["ship"]
            .as_object_mut()
            .ok_or_else(|| invalid("legacy ship source missing"))?;
        ship.entry("engines_enabled").or_insert(true.into());
        let p = ship
            .get_mut("propulsion")
            .and_then(serde_json::Value::as_object_mut)
            .ok_or_else(|| invalid("legacy propulsion missing"))?;
        if p.contains_key("forward_force_newtons") || p.contains_key("reverse_force_newtons") {
            for key in ["forward_force_newtons", "reverse_force_newtons"] {
                if !p
                    .get(key)
                    .and_then(serde_json::Value::as_f64)
                    .is_some_and(|v| v.is_finite() && v >= 0.0)
                {
                    return Err(invalid("legacy force configuration"));
                }
                p.remove(key);
            }
            p.insert("impulse_ns_per_joule".into(), 1.0.into());
        }
    }
    serde_json::to_vec(&value).map_err(|e| SaveError::Encoding(e.to_string()))
}
