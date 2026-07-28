use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::Cue;
use crate::dsp::envelope::gain_at;
use crate::dsp::noise::NoiseGenerator;
use crate::dsp::oscillator::{frequency_at, sample_at};
use crate::recipe::{Layer, NoiseLayer, ToneLayer, frames_for_ms, recipe};

const MIN_SAMPLE_RATE: u32 = 8_000;
const MAX_SAMPLE_RATE: u32 = 192_000;
const MAX_ABSOLUTE_PEAK: f32 = 0.9;

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

/// Renders `cue` as mono PCM into the required prefix of `output`.
///
/// Rendering performs no allocation. The returned value is the number of written frames; any
/// remaining tail is unchanged. Validation and capacity errors leave the entire buffer unchanged.
pub fn render_into(cue: Cue, sample_rate: u32, output: &mut [f32]) -> Result<usize, RenderError> {
    let required = required_frames(cue, sample_rate)?;
    if output.len() < required {
        return Err(RenderError::BufferTooSmall {
            required,
            provided: output.len(),
        });
    }

    output[..required].fill(0.0);

    for layer in recipe(cue).layers {
        match layer {
            Layer::Tone(layer) => render_tone(*layer, sample_rate, output),
            Layer::Noise(layer) => render_noise(*layer, sample_rate, output),
        }
    }

    for sample in &mut output[..required] {
        *sample = if sample.is_finite() {
            sample.clamp(-MAX_ABSOLUTE_PEAK, MAX_ABSOLUTE_PEAK)
        } else {
            0.0
        };
    }

    Ok(required)
}

fn render_tone(layer: ToneLayer, sample_rate: u32, output: &mut [f32]) {
    let start_frame = frames_for_ms(layer.offset_ms, sample_rate);
    let end_frame = frames_for_ms(layer.offset_ms + layer.envelope.duration_ms(), sample_rate);
    let mut phase = 0.0;

    for (local_frame, output_sample) in output[start_frame..end_frame].iter_mut().enumerate() {
        let time_ms = local_frame as f32 * 1_000.0 / sample_rate as f32;
        let frequency_hz = frequency_at(layer.frequency_hz, layer.glide, time_ms);
        let amplitude = gain_at(time_ms, layer.envelope) * layer.peak;
        *output_sample += sample_at(layer.waveform, phase) * amplitude;
        phase = (phase + frequency_hz / sample_rate as f32).fract();
    }
}

fn render_noise(layer: NoiseLayer, sample_rate: u32, output: &mut [f32]) {
    let start_frame = frames_for_ms(layer.offset_ms, sample_rate);
    let end_frame = frames_for_ms(layer.offset_ms + layer.envelope.duration_ms(), sample_rate);
    let mut generator = NoiseGenerator::new(layer.seed);

    for (local_frame, output_sample) in output[start_frame..end_frame].iter_mut().enumerate() {
        let time_ms = local_frame as f32 * 1_000.0 / sample_rate as f32;
        let amplitude = gain_at(time_ms, layer.envelope) * layer.peak;
        *output_sample += generator.next_sample() * amplitude;
    }
}

fn validate_sample_rate(sample_rate: u32) -> Result<(), RenderError> {
    if (MIN_SAMPLE_RATE..=MAX_SAMPLE_RATE).contains(&sample_rate) {
        Ok(())
    } else {
        Err(RenderError::UnsupportedSampleRate { sample_rate })
    }
}
