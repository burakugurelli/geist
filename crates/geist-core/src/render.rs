use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::Cue;
use crate::recipe::recipe;

const MIN_SAMPLE_RATE: u32 = 8_000;
const MAX_SAMPLE_RATE: u32 = 192_000;

/// Errors returned while preparing an interface sound for rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderError {
    /// The requested sample rate is outside the supported range.
    UnsupportedSampleRate {
        /// The unsupported sample rate in hertz.
        sample_rate: u32,
    },
    /// The caller-provided output buffer cannot contain all required frames.
    BufferTooSmall {
        /// The minimum number of frames required to render the cue.
        required: usize,
        /// The number of frames available in the caller-provided buffer.
        provided: usize,
    },
}

impl Display for RenderError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedSampleRate { sample_rate } => {
                write!(formatter, "unsupported sample rate: {sample_rate}")
            }
            Self::BufferTooSmall { required, provided } => {
                write!(
                    formatter,
                    "output buffer is too small: required {required}, provided {provided}",
                )
            }
        }
    }
}

impl Error for RenderError {}

/// Returns the number of frames required to render `cue` at `sample_rate`.
///
/// Supported sample rates are inclusive `8_000..=192_000 Hz`; required frames round upward to
/// a whole frame. The current possible error is [`RenderError::UnsupportedSampleRate`].
pub fn required_frames(cue: Cue, sample_rate: u32) -> Result<usize, RenderError> {
    validate_sample_rate(sample_rate)?;
    Ok(recipe(cue).required_frames(sample_rate))
}

fn validate_sample_rate(sample_rate: u32) -> Result<(), RenderError> {
    if (MIN_SAMPLE_RATE..=MAX_SAMPLE_RATE).contains(&sample_rate) {
        Ok(())
    } else {
        Err(RenderError::UnsupportedSampleRate { sample_rate })
    }
}
