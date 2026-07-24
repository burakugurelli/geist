use std::f32::consts::TAU;

use crate::recipe::{Glide, GlideCurve, Waveform};

pub(crate) fn sample_at(waveform: Waveform, phase: f32) -> f32 {
    match waveform {
        Waveform::Sine => (TAU * phase).sin(),
        Waveform::Triangle => 2.0 * (2.0 * (phase - (phase + 0.5).floor())).abs() - 1.0,
    }
}

pub(crate) fn frequency_at(start_hz: f32, glide: Option<Glide>, time_ms: f32) -> f32 {
    if !start_hz.is_finite() || start_hz <= 0.0 {
        return 0.0;
    }

    let Some(glide) = glide else {
        return start_hz;
    };

    if !glide.end_hz.is_finite() || glide.end_hz <= 0.0 {
        return start_hz;
    }

    if glide.duration_ms == 0 {
        return glide.end_hz;
    }

    if !time_ms.is_finite() {
        return start_hz;
    }

    let progress = time_ms / glide.duration_ms as f32;
    if !progress.is_finite() {
        return start_hz;
    }
    let progress = progress.clamp(0.0, 1.0);

    let frequency_hz = match glide.curve {
        GlideCurve::Linear => start_hz + (glide.end_hz - start_hz) * progress,
        GlideCurve::Exponential => start_hz * (glide.end_hz / start_hz).powf(progress),
    };

    if frequency_hz.is_finite() {
        frequency_hz
    } else {
        start_hz
    }
}

#[cfg(test)]
mod tests {
    use super::{frequency_at, sample_at};
    use crate::recipe::{Glide, GlideCurve, Waveform};

    #[test]
    fn waveforms_are_normalized() {
        for waveform in [Waveform::Sine, Waveform::Triangle] {
            for step in 0..1_000 {
                let phase = step as f32 / 1_000.0;
                assert!((-1.0..=1.0).contains(&sample_at(waveform, phase)));
            }
        }
    }

    #[test]
    fn sine_matches_canonical_phase_anchors() {
        let anchors = [
            (0.0, 0.0),
            (0.25, 1.0),
            (0.5, 0.0),
            (0.75, -1.0),
            (1.0, 0.0),
        ];

        for (phase, expected) in anchors {
            assert!((sample_at(Waveform::Sine, phase) - expected).abs() < 0.001);
        }
    }

    #[test]
    fn triangle_matches_canonical_phase_anchors() {
        let anchors = [
            (0.0, -1.0),
            (0.25, 0.0),
            (0.5, 1.0),
            (0.75, 0.0),
            (1.0, -1.0),
        ];

        for (phase, expected) in anchors {
            assert!((sample_at(Waveform::Triangle, phase) - expected).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn linear_glide_reaches_its_midpoint() {
        let glide = Glide {
            end_hz: 200.0,
            duration_ms: 100,
            curve: GlideCurve::Linear,
        };

        assert!((frequency_at(100.0, Some(glide), 50.0) - 150.0).abs() < 0.001);
    }

    #[test]
    fn exponential_glide_reaches_its_geometric_midpoint() {
        let glide = Glide {
            end_hz: 400.0,
            duration_ms: 100,
            curve: GlideCurve::Exponential,
        };

        assert!((frequency_at(100.0, Some(glide), 50.0) - 200.0).abs() < 0.001);
    }

    #[test]
    fn no_glide_returns_the_start_frequency() {
        assert_eq!(frequency_at(100.0, None, f32::NAN), 100.0);
    }

    #[test]
    fn glide_returns_start_end_and_end_after_its_duration() {
        let glide = Glide {
            end_hz: 200.0,
            duration_ms: 100,
            curve: GlideCurve::Linear,
        };

        assert_eq!(frequency_at(100.0, Some(glide), 0.0), 100.0);
        assert_eq!(frequency_at(100.0, Some(glide), 100.0), 200.0);
        assert_eq!(frequency_at(100.0, Some(glide), 150.0), 200.0);
    }

    #[test]
    fn zero_duration_glide_returns_its_end_immediately() {
        for curve in [GlideCurve::Linear, GlideCurve::Exponential] {
            let glide = Glide {
                end_hz: 200.0,
                duration_ms: 0,
                curve,
            };

            assert_eq!(frequency_at(100.0, Some(glide), 0.0), 200.0);
        }
    }

    #[test]
    fn invalid_start_frequencies_return_zero_frequency() {
        for start_hz in [0.0, -1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert_eq!(frequency_at(start_hz, None, 0.0), 0.0);
        }
    }

    #[test]
    fn invalid_glide_end_falls_back_to_start() {
        for end_hz in [0.0, -1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let glide = Glide {
                end_hz,
                duration_ms: 100,
                curve: GlideCurve::Linear,
            };

            assert_eq!(frequency_at(100.0, Some(glide), 50.0), 100.0);
        }
    }

    #[test]
    fn invalid_time_or_calculation_falls_back_to_start() {
        let glide = Glide {
            end_hz: 200.0,
            duration_ms: 100,
            curve: GlideCurve::Linear,
        };
        let overflowing_glide = Glide {
            end_hz: f32::MAX,
            duration_ms: 100,
            curve: GlideCurve::Exponential,
        };

        assert_eq!(frequency_at(100.0, Some(glide), f32::NAN), 100.0);
        assert_eq!(frequency_at(100.0, Some(glide), f32::INFINITY), 100.0);
        assert_eq!(
            frequency_at(f32::MIN_POSITIVE, Some(overflowing_glide), 50.0),
            f32::MIN_POSITIVE
        );
    }
}
