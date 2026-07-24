#![allow(
    dead_code,
    reason = "recipe fields are consumed by the renderer in the next task"
)]

use crate::Cue;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Waveform {
    Sine,
    Triangle,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum GlideCurve {
    Linear,
    Exponential,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Glide {
    pub(crate) end_hz: f32,
    pub(crate) duration_ms: u32,
    pub(crate) curve: GlideCurve,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Envelope {
    pub(crate) attack_ms: u32,
    pub(crate) decay_ms: u32,
}

impl Envelope {
    pub(crate) const fn duration_ms(self) -> u32 {
        self.attack_ms + self.decay_ms
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ToneLayer {
    pub(crate) offset_ms: u32,
    pub(crate) waveform: Waveform,
    pub(crate) frequency_hz: f32,
    pub(crate) envelope: Envelope,
    pub(crate) peak: f32,
    pub(crate) glide: Option<Glide>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct NoiseLayer {
    pub(crate) offset_ms: u32,
    pub(crate) envelope: Envelope,
    pub(crate) peak: f32,
    pub(crate) seed: u32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Layer {
    Tone(ToneLayer),
    Noise(NoiseLayer),
}

impl Layer {
    pub(crate) const fn end_ms(self) -> u32 {
        match self {
            Self::Tone(layer) => layer.offset_ms + layer.envelope.duration_ms(),
            Self::Noise(layer) => layer.offset_ms + layer.envelope.duration_ms(),
        }
    }
}

pub(crate) struct Recipe {
    pub(crate) layers: &'static [Layer],
}

impl Recipe {
    pub(crate) fn required_frames(&self, sample_rate: u32) -> usize {
        let duration_ms = self
            .layers
            .iter()
            .copied()
            .map(Layer::end_ms)
            .max()
            .unwrap_or_default();
        frames_for_ms(duration_ms, sample_rate)
    }
}

pub(crate) fn frames_for_ms(milliseconds: u32, sample_rate: u32) -> usize {
    let numerator = u64::from(milliseconds) * u64::from(sample_rate);
    numerator.div_ceil(1_000) as usize
}

const PRESS_LAYERS: [Layer; 2] = [
    Layer::Noise(NoiseLayer {
        offset_ms: 0,
        envelope: Envelope {
            attack_ms: 1,
            decay_ms: 35,
        },
        peak: 0.16,
        seed: 0xC0DE_0001,
    }),
    Layer::Tone(ToneLayer {
        offset_ms: 0,
        waveform: Waveform::Triangle,
        frequency_hz: 180.0,
        envelope: Envelope {
            attack_ms: 1,
            decay_ms: 45,
        },
        peak: 0.08,
        glide: Some(Glide {
            end_hz: 120.0,
            duration_ms: 45,
            curve: GlideCurve::Exponential,
        }),
    }),
];

const SUCCESS_LAYERS: [Layer; 3] = [
    Layer::Tone(ToneLayer {
        offset_ms: 0,
        waveform: Waveform::Sine,
        frequency_hz: 659.25,
        envelope: Envelope {
            attack_ms: 4,
            decay_ms: 120,
        },
        peak: 0.14,
        glide: None,
    }),
    Layer::Tone(ToneLayer {
        offset_ms: 65,
        waveform: Waveform::Sine,
        frequency_hz: 830.61,
        envelope: Envelope {
            attack_ms: 4,
            decay_ms: 140,
        },
        peak: 0.13,
        glide: None,
    }),
    Layer::Tone(ToneLayer {
        offset_ms: 130,
        waveform: Waveform::Sine,
        frequency_hz: 987.77,
        envelope: Envelope {
            attack_ms: 4,
            decay_ms: 200,
        },
        peak: 0.15,
        glide: None,
    }),
];

const ERROR_LAYERS: [Layer; 3] = [
    Layer::Noise(NoiseLayer {
        offset_ms: 0,
        envelope: Envelope {
            attack_ms: 1,
            decay_ms: 40,
        },
        peak: 0.08,
        seed: 0xC0DE_0002,
    }),
    Layer::Tone(ToneLayer {
        offset_ms: 20,
        waveform: Waveform::Triangle,
        frequency_hz: 392.0,
        envelope: Envelope {
            attack_ms: 4,
            decay_ms: 140,
        },
        peak: 0.12,
        glide: Some(Glide {
            end_hz: 329.63,
            duration_ms: 130,
            curve: GlideCurve::Linear,
        }),
    }),
    Layer::Tone(ToneLayer {
        offset_ms: 130,
        waveform: Waveform::Triangle,
        frequency_hz: 293.66,
        envelope: Envelope {
            attack_ms: 4,
            decay_ms: 180,
        },
        peak: 0.11,
        glide: None,
    }),
];

const PRESS: Recipe = Recipe {
    layers: &PRESS_LAYERS,
};
const SUCCESS: Recipe = Recipe {
    layers: &SUCCESS_LAYERS,
};
const ERROR: Recipe = Recipe {
    layers: &ERROR_LAYERS,
};

pub(crate) const fn recipe(cue: Cue) -> &'static Recipe {
    match cue {
        Cue::Press => &PRESS,
        Cue::Success => &SUCCESS,
        Cue::Error => &ERROR,
    }
}
