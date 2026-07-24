#![allow(
    dead_code,
    reason = "DSP primitives are consumed by the renderer in the next task"
)]

pub(crate) mod envelope;
pub(crate) mod noise;
pub(crate) mod oscillator;
