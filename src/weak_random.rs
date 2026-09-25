/// Implementation clone of
/// https://github.com/oven-sh/WebKit/blob/74650443cb1a41519624470b386c850c1927762b/Source/WTF/wtf/WeakRandom.h#L42

pub struct WeakRandom {
    seed: u32,
    low: u64,
    high: u64,
}

fn next_state(mut x: u64, y: u64) -> u64 {
    x ^= x << 23;
    x ^= x >> 17;
    x ^= y ^ (y >> 26);
    x
}

impl WeakRandom {
    pub fn from_seed(seed: u32) -> Self {
        let seed = seed.max(1); // zero seed would cause an infinite series of 0s.
        let mut rng = Self { seed, low: seed as u64, high: seed as u64 };
        rng.advance();
        rng
    }

    pub fn seed(&self) -> u32 {
        self.seed
    }

    pub fn get(&mut self) -> f64 {
        const MASK: u64 = (1 << 53) - 1;
        const SCALE: f64 = 1.0 / (1u64 << 53) as f64;
        let value = self.advance() & MASK;
        (value as f64) * SCALE
    }

    fn advance(&mut self) -> u64 {
        let x = self.low;
        let y = self.high;
        self.low = y;
        self.high = next_state(x, y);
        self.high + self.low
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seed_saved() {
        let rng = WeakRandom::from_seed(1337);
        assert!(rng.seed() == 1337)
    }

    #[test]
    fn test_seed_1337_generates_matches_bun() {
        let mut rng = WeakRandom::from_seed(1337);
        // Output generated via:
        // bun -e 'import { setRandomSeed } from "bun:jsc"; setRandomSeed(1337); console.log(Math.random())'
        assert!(rng.get() == 0.0000012451879418673428f64);
    }
}
