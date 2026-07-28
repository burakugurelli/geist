use geist_core::{Cue, render_into, required_frames};
use wasm_bindgen::prelude::*;

/// Renders a named interface sound as mono PCM samples.
#[wasm_bindgen]
pub fn render(cue_name: &str, sample_rate: u32) -> Result<Box<[f32]>, JsValue> {
    render_pcm(cue_name, sample_rate).map_err(|message| wasm_bindgen::JsError::new(&message).into())
}

fn render_pcm(cue_name: &str, sample_rate: u32) -> Result<Box<[f32]>, String> {
    let cue = Cue::try_from(cue_name).map_err(|error| error.to_string())?;
    let frame_count = required_frames(cue, sample_rate).map_err(|error| error.to_string())?;
    let mut samples = vec![0.0; frame_count];
    render_into(cue, sample_rate, &mut samples).map_err(|error| error.to_string())?;
    Ok(samples.into_boxed_slice())
}

#[cfg(test)]
mod tests {
    use super::render_pcm;

    #[test]
    fn every_known_cue_renders_finite_pcm_samples() {
        for cue_name in ["press", "success", "error"] {
            let samples = render_pcm(cue_name, 48_000).expect("known cue should render");

            assert!(!samples.is_empty(), "{cue_name} should not be empty");
            assert!(
                samples.iter().all(|sample| sample.is_finite()),
                "{cue_name} should contain only finite samples"
            );
        }
    }

    #[test]
    fn unknown_cue_returns_the_core_error_message() {
        assert_eq!(
            render_pcm("chime", 48_000),
            Err("unknown interface sound cue".to_owned())
        );
    }
}
