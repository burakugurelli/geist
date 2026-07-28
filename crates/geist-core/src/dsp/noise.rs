pub(crate) struct NoiseGenerator {
    state: u32,
}

impl NoiseGenerator {
    pub(crate) fn new(seed: u32) -> Self {
        Self { state: seed.max(1) }
    }

    pub(crate) fn next_sample(&mut self) -> f32 {
        // Xorshift32 keeps noise reproducible across every host platform.
        let mut value = self.state;
        value ^= value << 13;
        value ^= value >> 17;
        value ^= value << 5;
        self.state = value;

        (value as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::NoiseGenerator;

    #[test]
    fn equal_seeds_produce_equal_samples() {
        let mut first = NoiseGenerator::new(42);
        let mut second = NoiseGenerator::new(42);

        for _ in 0..32 {
            assert_eq!(first.next_sample(), second.next_sample());
        }
    }

    #[test]
    fn generated_samples_remain_normalized() {
        let mut generator = NoiseGenerator::new(42);

        for _ in 0..1_000 {
            assert!((-1.0..=1.0).contains(&generator.next_sample()));
        }
    }

    #[test]
    fn fixed_seed_matches_known_xorshift32_samples() {
        let mut generator = NoiseGenerator::new(42);
        let expected_bits = [0xBF7E_A576, 0x3EA4_28D4, 0xBF47_30A2, 0x3F32_E188];

        for bits in expected_bits {
            assert_eq!(generator.next_sample().to_bits(), bits);
        }
    }

    #[test]
    fn zero_seed_uses_the_seed_one_sequence() {
        let mut zero_seed = NoiseGenerator::new(0);
        let mut one_seed = NoiseGenerator::new(1);

        for _ in 0..32 {
            assert_eq!(zero_seed.next_sample(), one_seed.next_sample());
        }
    }

    #[test]
    fn distinct_seeds_produce_different_initial_samples() {
        assert_ne!(
            NoiseGenerator::new(42).next_sample().to_bits(),
            NoiseGenerator::new(43).next_sample().to_bits()
        );
    }

    #[test]
    fn nonconsecutive_samples_are_distinct() {
        let mut generator = NoiseGenerator::new(42);

        let first = generator.next_sample();
        let _ = generator.next_sample();
        let third = generator.next_sample();

        assert_ne!(first.to_bits(), third.to_bits());
    }
}
