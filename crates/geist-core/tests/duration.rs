use geist_core::{Cue, RenderError, required_frames};

#[test]
fn recipe_lengths_are_stable_at_48_khz() {
    assert_eq!(required_frames(Cue::Press, 48_000), Ok(2_208));
    assert_eq!(required_frames(Cue::Success, 48_000), Ok(16_032));
    assert_eq!(required_frames(Cue::Error, 48_000), Ok(15_072));
}

#[test]
fn common_browser_sample_rates_are_supported() {
    assert!(required_frames(Cue::Success, 44_100).is_ok());
    assert!(required_frames(Cue::Success, 48_000).is_ok());
}

#[test]
fn fractional_frame_counts_round_up() {
    assert_eq!(required_frames(Cue::Press, 44_100), Ok(2_029));
}

#[test]
fn unsafe_sample_rates_are_rejected() {
    assert_eq!(
        required_frames(Cue::Press, 0),
        Err(RenderError::UnsupportedSampleRate { sample_rate: 0 }),
    );
    assert_eq!(
        required_frames(Cue::Press, 7_999),
        Err(RenderError::UnsupportedSampleRate { sample_rate: 7_999 }),
    );
    assert_eq!(
        required_frames(Cue::Press, 192_001),
        Err(RenderError::UnsupportedSampleRate {
            sample_rate: 192_001,
        }),
    );
}

#[test]
fn sample_rate_boundaries_are_supported() {
    assert_eq!(required_frames(Cue::Press, 8_000), Ok(368));
    assert_eq!(required_frames(Cue::Press, 192_000), Ok(8_832));
}

#[test]
fn every_initial_cue_is_shorter_than_600_milliseconds() {
    for cue in Cue::ALL {
        let frames = required_frames(cue, 48_000).expect("supported sample rate");
        assert!(frames < 28_800, "{} is too long", cue.as_str());
    }
}
