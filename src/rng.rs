//! Tiny xorshift64 generator; plenty for picking flash cards.

use std::time::{SystemTime, UNIX_EPOCH};

pub struct Rng(u64);

impl Default for Rng {
    fn default() -> Self {
        Self::new()
    }
}

impl Rng {
    pub fn new() -> Self {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(88172645463325252);
        Rng::seeded(n)
    }

    pub fn seeded(seed: u64) -> Self {
        Rng(seed | 1)
    }

    /// Uniform in [0, 1).
    pub fn next_f64(&mut self) -> f64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        (x >> 11) as f64 / (1u64 << 53) as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stays_in_unit_interval() {
        let mut r = Rng::seeded(42);
        for _ in 0..10_000 {
            let v = r.next_f64();
            assert!((0.0..1.0).contains(&v));
        }
    }

    #[test]
    fn same_seed_same_sequence() {
        let (mut a, mut b) = (Rng::seeded(7), Rng::seeded(7));
        for _ in 0..100 {
            assert_eq!(a.next_f64(), b.next_f64());
        }
    }

    #[test]
    fn zero_seed_does_not_stick() {
        let mut r = Rng::seeded(0);
        assert_ne!(r.next_f64(), r.next_f64());
    }
}
