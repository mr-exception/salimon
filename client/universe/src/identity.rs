use crate::{Error, invalid};
use serde::{Deserialize, Serialize};

/// Exact source namespace/key identity; never a display name, row index or hash.
/// Namespace is lowercase ASCII; keys retain source case. No lossy normalization.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct CatalogId(String);
impl CatalogId {
    pub fn from_source(namespace: &str, key: &str) -> Result<Self, Error> {
        Self::try_from(format!("{namespace}:{key}"))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl TryFrom<String> for CatalogId {
    type Error = Error;
    fn try_from(value: String) -> Result<Self, Error> {
        let Some((namespace, key)) = value.split_once(':') else {
            return Err(invalid("identity requires namespace:key"));
        };
        let atom = |s: &str| {
            !s.is_empty()
                && s.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        };
        if value.len() > 160
            || !atom(namespace)
            || namespace.bytes().any(|b| b.is_ascii_uppercase())
            || !atom(key)
        {
            return Err(invalid(
                "identity requires bounded ASCII namespace/key without whitespace",
            ));
        }
        Ok(Self(value))
    }
}
impl From<CatalogId> for String {
    fn from(id: CatalogId) -> Self {
        id.0
    }
}
