use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "Vec<PriorityRow>", into = "Vec<PriorityRow>")]
pub struct PrioritySidecar(Vec<PriorityRow>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PriorityRow {
    link: usize,
    priority: f64,
}

#[derive(Debug)]
pub enum PriorityError {
    OutOfRange,
    DuplicateLink,
}

impl std::fmt::Display for PriorityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::OutOfRange => "priority must be finite and within -0.20..=0.20",
            Self::DuplicateLink => "duplicate priority link index",
        })
    }
}

impl std::error::Error for PriorityError {}

impl TryFrom<Vec<PriorityRow>> for PrioritySidecar {
    type Error = PriorityError;

    fn try_from(mut rows: Vec<PriorityRow>) -> Result<Self, Self::Error> {
        rows.sort_by_key(|row| row.link);
        if rows
            .iter()
            .any(|row| !row.priority.is_finite() || !(-0.2..=0.2).contains(&row.priority))
        {
            return Err(PriorityError::OutOfRange);
        }
        if rows.windows(2).any(|pair| pair[0].link == pair[1].link) {
            return Err(PriorityError::DuplicateLink);
        }
        Ok(Self(rows))
    }
}

impl From<PrioritySidecar> for Vec<PriorityRow> {
    fn from(value: PrioritySidecar) -> Self {
        value.0
    }
}

impl PrioritySidecar {
    pub fn validate_links(&self, final_count: usize) -> Result<()> {
        ensure!(
            self.0.iter().all(|row| row.link < final_count),
            "priority link index outside final pool"
        );
        Ok(())
    }

    pub fn priority(&self, link: usize) -> Option<f64> {
        self.0
            .iter()
            .find(|row| row.link == link)
            .map(|row| row.priority)
    }
}
