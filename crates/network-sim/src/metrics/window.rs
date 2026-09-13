use serde::{Deserialize, Serialize};

use super::MetricError;

/// Signed run-relative milliseconds allow the ten-second pre-measurement baseline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "WindowFields")]
pub struct Window {
    start_ms: i64,
    end_ms: i64,
}

#[derive(Deserialize)]
struct WindowFields {
    start_ms: i64,
    end_ms: i64,
}

impl TryFrom<WindowFields> for Window {
    type Error = MetricError;
    fn try_from(value: WindowFields) -> Result<Self, Self::Error> {
        Self::new(value.start_ms, value.end_ms)
    }
}

impl Window {
    pub fn new(start_ms: i64, end_ms: i64) -> Result<Self, MetricError> {
        match end_ms.checked_sub(start_ms) {
            Some(duration) if duration > 0 => Ok(Self { start_ms, end_ms }),
            Some(_) | None => Err(MetricError::InvalidWindow),
        }
    }

    pub const fn start_ms(self) -> i64 {
        self.start_ms
    }
    pub const fn end_ms(self) -> i64 {
        self.end_ms
    }
    pub const fn duration_ms(self) -> i64 {
        self.end_ms - self.start_ms
    }
    pub const fn contains(self, t_ms: i64) -> bool {
        t_ms > self.start_ms && t_ms <= self.end_ms
    }
}
