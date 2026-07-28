use crate::recipe::Envelope;

pub(crate) fn gain_at(time_ms: f32, envelope: Envelope) -> f32 {
    let attack_ms = envelope.attack_ms as f32;
    if time_ms < attack_ms {
        return if attack_ms == 0.0 {
            1.0
        } else {
            time_ms / attack_ms
        };
    }

    let decay_ms = envelope.decay_ms as f32;
    let decay_time_ms = time_ms - attack_ms;
    if decay_ms == 0.0 || decay_time_ms >= decay_ms {
        0.0
    } else {
        1.0 - decay_time_ms / decay_ms
    }
}

#[cfg(test)]
mod tests {
    use super::gain_at;
    use crate::recipe::Envelope;

    const ENVELOPE: Envelope = Envelope {
        attack_ms: 10,
        decay_ms: 90,
    };

    #[test]
    fn attack_and_decay_reach_expected_points() {
        assert_eq!(gain_at(0.0, ENVELOPE), 0.0);
        assert!((gain_at(5.0, ENVELOPE) - 0.5).abs() < f32::EPSILON);
        assert_eq!(gain_at(10.0, ENVELOPE), 1.0);
        assert!((gain_at(55.0, ENVELOPE) - 0.5).abs() < f32::EPSILON);
        assert_eq!(gain_at(100.0, ENVELOPE), 0.0);
    }

    #[test]
    fn zero_attack_starts_at_full_gain() {
        let envelope = Envelope {
            attack_ms: 0,
            decay_ms: 10,
        };

        assert_eq!(gain_at(0.0, envelope), 1.0);
        assert!((gain_at(5.0, envelope) - 0.5).abs() < f32::EPSILON);
        assert_eq!(gain_at(10.0, envelope), 0.0);
    }

    #[test]
    fn zero_decay_ends_at_the_attack_boundary() {
        let envelope = Envelope {
            attack_ms: 10,
            decay_ms: 0,
        };

        assert_eq!(gain_at(0.0, envelope), 0.0);
        assert!((gain_at(5.0, envelope) - 0.5).abs() < f32::EPSILON);
        assert_eq!(gain_at(10.0, envelope), 0.0);
    }

    #[test]
    fn zero_attack_and_decay_has_no_duration() {
        let envelope = Envelope {
            attack_ms: 0,
            decay_ms: 0,
        };

        assert_eq!(gain_at(0.0, envelope), 0.0);
    }
}
