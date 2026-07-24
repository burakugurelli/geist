use geist_core::{Cue, RenderError, render_into, required_frames};

const SAMPLE_RATE: u32 = 48_000;

fn render(cue: Cue) -> Vec<f32> {
    let frame_count = required_frames(cue, SAMPLE_RATE).expect("valid recipe");
    let mut output = vec![0.0; frame_count];
    let written = render_into(cue, SAMPLE_RATE, &mut output).expect("render succeeds");

    assert_eq!(written, frame_count);
    output
}

fn assert_pcm_differs(first: &[f32], second: &[f32]) {
    let common_frame_count = first.len().min(second.len());

    assert!(common_frame_count > 0);
    assert!(
        first[..common_frame_count]
            .iter()
            .zip(&second[..common_frame_count])
            .any(|(first_sample, second_sample)| first_sample.to_bits() != second_sample.to_bits())
    );
}

#[test]
fn every_cue_is_deterministic_finite_and_audible() {
    for cue in Cue::ALL {
        let first = render(cue);
        let second = render(cue);

        assert_eq!(first, second, "{} is not deterministic", cue.as_str());
        assert!(first.iter().all(|sample| sample.is_finite()));
        assert!(first.iter().any(|sample| sample.abs() > 0.000_1));
        assert!(first.iter().all(|sample| sample.abs() <= 0.9));
    }
}

#[test]
fn cues_have_distinct_sample_data() {
    assert_pcm_differs(&render(Cue::Press), &render(Cue::Success));
    assert_pcm_differs(&render(Cue::Success), &render(Cue::Error));
    assert_pcm_differs(&render(Cue::Press), &render(Cue::Error));
}

#[test]
fn a_small_buffer_is_not_partially_modified() {
    let required = required_frames(Cue::Success, SAMPLE_RATE).expect("valid recipe");
    let mut output = vec![0.25; required - 1];

    assert_eq!(
        render_into(Cue::Success, SAMPLE_RATE, &mut output),
        Err(RenderError::BufferTooSmall {
            required,
            provided: required - 1,
        }),
    );
    assert!(output.iter().all(|sample| *sample == 0.25));
}

#[test]
fn an_unsupported_sample_rate_does_not_modify_the_buffer() {
    let sample_rate = 7_999;
    let mut output = vec![0.25; 16];
    let original = output.clone();

    assert_eq!(
        render_into(Cue::Press, sample_rate, &mut output),
        Err(RenderError::UnsupportedSampleRate { sample_rate }),
    );
    assert_eq!(output, original);
}

#[test]
fn an_oversized_buffer_keeps_its_tail_unchanged() {
    let required = required_frames(Cue::Error, SAMPLE_RATE).expect("valid recipe");
    let mut output = vec![0.25; required + 16];

    assert_eq!(
        render_into(Cue::Error, SAMPLE_RATE, &mut output),
        Ok(required)
    );
    assert!(output[required..].iter().all(|sample| *sample == 0.25));
}
