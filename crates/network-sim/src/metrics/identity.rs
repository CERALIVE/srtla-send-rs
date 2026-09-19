use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::MetricError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Hash256(String);

impl TryFrom<String> for Hash256 {
    type Error = MetricError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() != 64 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(MetricError::InvalidField("sha256".into()));
        }
        Ok(Self(value.to_ascii_lowercase()))
    }
}

impl From<Hash256> for String {
    fn from(value: Hash256) -> Self {
        value.0
    }
}

impl Hash256 {
    pub fn digest(bytes: &[u8]) -> Self {
        Self(
            Sha256::digest(bytes)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct RunId(String);

impl TryFrom<String> for RunId {
    type Error = MetricError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() != 36
            || !value.bytes().enumerate().all(|(i, b)| {
                if [8, 13, 18, 23].contains(&i) {
                    b == b'-'
                } else {
                    b.is_ascii_hexdigit()
                }
            })
        {
            return Err(MetricError::InvalidField("run_id UUID".into()));
        }
        Ok(Self(value.to_ascii_lowercase()))
    }
}

impl From<RunId> for String {
    fn from(value: RunId) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct SchemaVersion;

impl TryFrom<u8> for SchemaVersion {
    type Error = MetricError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self),
            _ => Err(MetricError::InvalidField("RunRecord schema_version".into())),
        }
    }
}

impl From<SchemaVersion> for u8 {
    fn from(_: SchemaVersion) -> Self {
        1
    }
}
