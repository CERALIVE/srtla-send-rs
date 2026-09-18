use std::error::Error;
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SchedulingMode {
    #[default]
    Enhanced,
}

impl SchedulingMode {
    pub const fn as_u8(self) -> u8 {
        match self {
            Self::Enhanced => 1,
        }
    }

    pub const fn from_u8(_value: u8) -> Self {
        Self::Enhanced
    }

    pub const fn is_enhanced(self) -> bool {
        matches!(self, Self::Enhanced)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModeError {
    Retired,
    Unknown,
}

impl fmt::Display for ModeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Retired => {
                f.write_str("mode removed in 4.0.0; use enhanced; see docs/release-notes-4.0.0.md")
            }
            Self::Unknown => {
                f.write_str("invalid mode; use enhanced; see docs/release-notes-4.0.0.md")
            }
        }
    }
}

impl Error for ModeError {}

#[derive(Clone)]
pub struct ModeParser;

impl clap::builder::TypedValueParser for ModeParser {
    type Value = SchedulingMode;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        _arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        value
            .to_str()
            .ok_or(ModeError::Unknown)
            .and_then(str::parse)
            .map_err(|error| {
                clap::Error::raw(
                    clap::error::ErrorKind::InvalidValue,
                    format!("invalid value {value:?} for --mode: {error}"),
                )
                .with_cmd(cmd)
            })
    }

    fn possible_values(
        &self,
    ) -> Option<Box<dyn Iterator<Item = clap::builder::PossibleValue> + '_>> {
        Some(Box::new(std::iter::once(
            clap::builder::PossibleValue::new("enhanced"),
        )))
    }
}

impl fmt::Display for SchedulingMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Enhanced => f.write_str("enhanced"),
        }
    }
}

impl FromStr for SchedulingMode {
    type Err = ModeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "enhanced" => Ok(Self::Enhanced),
            "classic" | "rtt-threshold" | "edpf" | "adaptive" => Err(ModeError::Retired),
            _ => Err(ModeError::Unknown),
        }
    }
}

impl clap::ValueEnum for SchedulingMode {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Enhanced]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self {
            Self::Enhanced => Some(clap::builder::PossibleValue::new("enhanced")),
        }
    }

    fn from_str(input: &str, _ignore_case: bool) -> Result<Self, String> {
        input.parse().map_err(|error: ModeError| error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enhanced_retains_its_encoding_and_default() {
        let mode = SchedulingMode::default();
        assert_eq!(mode.as_u8(), 1);
        assert_eq!(SchedulingMode::from_u8(mode.as_u8()), mode);
        assert_eq!(mode.to_string(), "enhanced");
        assert_eq!("enhanced".parse(), Ok(mode));
        for byte in 0..=u8::MAX {
            assert_eq!(SchedulingMode::from_u8(byte), mode);
        }
    }

    #[test]
    fn deleted_spellings_have_a_distinct_typed_error() {
        for name in ["classic", "rtt-threshold", "edpf", "adaptive"] {
            assert_eq!(name.parse::<SchedulingMode>(), Err(ModeError::Retired));
        }
        assert_eq!("other".parse::<SchedulingMode>(), Err(ModeError::Unknown));
    }
}
