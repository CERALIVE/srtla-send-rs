use std::error::Error;
use std::{fmt, io};

#[derive(Debug)]
pub enum MetricError {
    MissingColumn(String),
    InvalidField(String),
    CounterReset(String),
    MissingBaseline,
    InvalidWindow,
    IncompleteSeries,
    Io(io::Error),
    Json(serde_json::Error),
}

impl fmt::Display for MetricError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingColumn(name) => write!(f, "missing column {name}"),
            Self::InvalidField(name) => write!(f, "invalid field {name}"),
            Self::CounterReset(name) => write!(
                f,
                "counter reset in {name}; check capture semantics or process generation"
            ),
            Self::MissingBaseline => f.write_str("no CSV row at or before window start"),
            Self::InvalidWindow => {
                f.write_str("window must have a positive, representable duration")
            }
            Self::IncompleteSeries => {
                f.write_str("missing, overlapping, or unordered measurement buckets")
            }
            Self::Io(error) => error.fmt(f),
            Self::Json(error) => error.fmt(f),
        }
    }
}

impl Error for MetricError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Json(error) => Some(error),
            Self::MissingColumn(_)
            | Self::InvalidField(_)
            | Self::CounterReset(_)
            | Self::MissingBaseline
            | Self::InvalidWindow
            | Self::IncompleteSeries => None,
        }
    }
}

impl From<io::Error> for MetricError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for MetricError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

pub(super) fn number<T: std::str::FromStr>(value: &str, field: &str) -> Result<T, MetricError> {
    value
        .trim()
        .parse()
        .map_err(|_| MetricError::InvalidField(field.into()))
}

pub(super) fn delta(end: u64, start: u64, field: &str) -> Result<u64, MetricError> {
    end.checked_sub(start)
        .ok_or_else(|| MetricError::CounterReset(field.into()))
}
