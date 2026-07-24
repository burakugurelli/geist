use std::error::Error;
use std::fmt::{Display, Formatter};

/// A semantic interface sound supplied by the built-in palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cue {
    Press,
    Success,
    Error,
}

impl Cue {
    pub const ALL: [Self; 3] = [Self::Press, Self::Success, Self::Error];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Press => "press",
            Self::Success => "success",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownCue;

impl Display for UnknownCue {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("unknown interface sound cue")
    }
}

impl Error for UnknownCue {}

impl TryFrom<&str> for Cue {
    type Error = UnknownCue;

    fn try_from(value: &str) -> Result<Self, UnknownCue> {
        match value {
            "press" => Ok(Self::Press),
            "success" => Ok(Self::Success),
            "error" => Ok(Self::Error),
            _ => Err(UnknownCue),
        }
    }
}
