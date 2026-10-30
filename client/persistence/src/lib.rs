//! Canonical envelope. Domain payloads implement validation before publication.
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{fmt, time::Duration};

pub const SCHEMA_VERSION: u32 = 28;
pub const FORMAT: &str = "salimon.save";
pub const MAX_SAVE_BYTES: usize = 64 * 1024 * 1024;

#[derive(Debug, PartialEq, Eq)]
pub enum SaveError {
    Encoding(String),
    UnsupportedVersion(u32),
    /// Valid bytes bound to a different immutable universe; never auto-reseed.
    IncompatibleCatalog(String),
    Invalid(String),
}
impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "save contract: {self:?}")
    }
}
impl std::error::Error for SaveError {}

pub trait Validate {
    fn validate(&self) -> Result<(), SaveError>;
}

/// Wall time is Unix seconds, never an Instant. Simulation/backlog use exact
/// Duration seconds/nanoseconds. Closed-app world advancement is always zero;
/// a future fatigue owner alone may use the nonnegative wall-time difference.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveGame<T> {
    pub format: String,
    pub schema_version: u32,
    pub saved_at_unix_seconds: u64,
    pub simulated_time: Duration,
    pub pending_time: Duration,
    pub state: T,
}
impl<T: Validate> SaveGame<T> {
    pub fn new(
        state: T,
        simulated_time: Duration,
        pending_time: Duration,
        now: u64,
    ) -> Result<Self, SaveError> {
        state.validate()?;
        Ok(Self {
            format: FORMAT.into(),
            schema_version: SCHEMA_VERSION,
            saved_at_unix_seconds: now,
            simulated_time,
            pending_time,
            state,
        })
    }
    pub fn validate(&self) -> Result<(), SaveError> {
        if !matches!(self.schema_version, 1..=SCHEMA_VERSION) {
            return Err(SaveError::UnsupportedVersion(self.schema_version));
        }
        if self.format != FORMAT {
            return Err(invalid("unrecognized format"));
        }
        self.state.validate()
    }
}
impl<T: Validate + Serialize + DeserializeOwned> SaveGame<T> {
    pub fn encode(&self) -> Result<Vec<u8>, SaveError> {
        self.validate()?;
        let bytes = serde_json::to_vec(self).map_err(|e| SaveError::Encoding(e.to_string()))?;
        if bytes.len() > MAX_SAVE_BYTES {
            return Err(invalid("save exceeds size limit"));
        }
        Ok(bytes)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, SaveError> {
        if bytes.len() > MAX_SAVE_BYTES {
            return Err(invalid("save exceeds size limit"));
        }
        // Inspect the version before decoding any domain state. Runtime owns
        // the explicit v1/v2/v3/v4 -> v5 payload migration; unknown versions reject.
        #[derive(Deserialize)]
        struct Header {
            schema_version: u32,
        }
        let header: Header =
            serde_json::from_slice(bytes).map_err(|e| SaveError::Encoding(e.to_string()))?;
        if !matches!(header.schema_version, 1..=SCHEMA_VERSION) {
            return Err(SaveError::UnsupportedVersion(header.schema_version));
        }
        let save: Self =
            serde_json::from_slice(bytes).map_err(|e| SaveError::Encoding(e.to_string()))?;
        save.validate()?;
        Ok(save)
    }
}
pub fn invalid(message: impl Into<String>) -> SaveError {
    SaveError::Invalid(message.into())
}

/// JSON object keys cannot represent stable structured IDs. Encode maps as
/// ordered pairs, reject duplicates, and sort by their canonical serialized key.
pub mod pairs {
    use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};
    use std::{collections::HashMap, hash::Hash};
    pub fn serialize<K: Serialize, V: Serialize, S: Serializer>(
        map: &HashMap<K, V>,
        s: S,
    ) -> Result<S::Ok, S::Error> {
        let mut entries: Vec<_> = map
            .iter()
            .map(|(k, v)| {
                Ok((
                    serde_json::to_string(k).map_err(serde::ser::Error::custom)?,
                    k,
                    v,
                ))
            })
            .collect::<Result<_, S::Error>>()?;
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        entries
            .into_iter()
            .map(|(_, k, v)| (k, v))
            .collect::<Vec<_>>()
            .serialize(s)
    }
    pub fn deserialize<
        'de,
        K: Deserialize<'de> + Eq + Hash,
        V: Deserialize<'de>,
        D: Deserializer<'de>,
    >(
        d: D,
    ) -> Result<HashMap<K, V>, D::Error> {
        let entries = Vec::<(K, V)>::deserialize(d)?;
        let mut map = HashMap::with_capacity(entries.len());
        for (k, v) in entries {
            if map.insert(k, v).is_some() {
                return Err(D::Error::custom("duplicate stable identity"));
            }
        }
        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    #[serde(deny_unknown_fields)]
    struct Fixture {
        seed: u64,
        charge: f64,
    }
    impl Validate for Fixture {
        fn validate(&self) -> Result<(), SaveError> {
            if self.charge.is_finite() && self.charge >= 0. {
                Ok(())
            } else {
                Err(invalid("charge"))
            }
        }
    }
    #[test]
    fn exact_clock_and_float_roundtrip_and_version_rejection() {
        let save = SaveGame::new(
            Fixture {
                seed: u64::MAX,
                charge: 0.12345678901234568,
            },
            Duration::new(123, 456),
            Duration::new(2, 999),
            42,
        )
        .unwrap();
        let bytes = save.encode().unwrap();
        let restored = SaveGame::<Fixture>::decode(&bytes).unwrap();
        assert_eq!(restored.state, save.state);
        assert_eq!(restored.simulated_time, save.simulated_time);
        assert_eq!(restored.pending_time, save.pending_time);
        let future = String::from_utf8(bytes).unwrap().replace(
            &format!("\"schema_version\":{SCHEMA_VERSION}"),
            &format!("\"schema_version\":{}", SCHEMA_VERSION + 1),
        );
        assert_eq!(
            SaveGame::<Fixture>::decode(future.as_bytes()).unwrap_err(),
            SaveError::UnsupportedVersion(SCHEMA_VERSION + 1)
        );
        assert!(SaveGame::<Fixture>::decode(b"{}").is_err());
        assert!(
            SaveGame::new(
                Fixture {
                    seed: 0,
                    charge: -1.
                },
                Duration::ZERO,
                Duration::ZERO,
                0
            )
            .is_err()
        );
    }
}

pub mod storage;
pub mod universe;
