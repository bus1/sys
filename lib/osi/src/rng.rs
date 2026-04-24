//! # Random Number Generators
//!
//! This module provides implementations of common random number generators
//! (RNGs). Only pseudo-random number generators (PRNGs) are provided for now.
//!
//! Depending on the use-case, some RNGs might be inapplicable (e.g., some are
//! **not** cryptographically secure). Choose the right RNG for each use-case
//! carefully.

/// Random number generator using SplitMix64.
///
/// This is the state of the SplitMix64 pseudo random number generator. It uses
/// 64-bit of state and generates 64-bit random numbers. This PRNG is **not**
/// cryptographically secure.
///
/// This RNG can be split, but will produce instances of [`SplitMix64`]. A
/// split RNG retains certain probabilistic properties across the entire
/// (possibly recursive) set of split RNGs. Split RNGs should be preferred
/// over creating multiple independent instances with independent seeds.
///
/// This implements `Clone` and `Copy` for verbatim copies. Use
/// [`split()`](Self::split) to produce non-verbatim copies, but with better
/// random distribution.
///
/// For details see its
/// [research article](http://dx.doi.org/10.1145/2714064.2660195)
/// or the documentation of
/// [Java SplittableRandom](http://docs.oracle.com/javase/8/docs/api/java/util/SplittableRandom.html)).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Mix64 {
    state: u64,
}

/// Split random number generator using SplitMix64.
///
/// This is the same as [`Mix64`] but was split off another RNG. This requires
/// 128-bit of state, compared to 64-bit for `Mix64`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SplitMix64 {
    mix: Mix64,
    gamma: u64,
}

impl Mix64 {
    const GAMMA: u64 = 0x9e3779b97f4a7c15;

    // Computes the 32 high bits of Stafford variant 4 mix64 function:
    // - http://zimbry.blogspot.com/2011/09/better-bit-mixing-improving-on.html
    fn mix32(mut v: u64) -> u32 {
        v = (v ^ (v >> 33)).wrapping_mul(0x62a9d9ed799705f5);
        v = (v ^ (v >> 28)).wrapping_mul(0xcb24d0a5c88c35b3);
        (v >> 32) as u32
    }

    // Computes Stafford variant 13 mix64 function:
    // - http://zimbry.blogspot.com/2011/09/better-bit-mixing-improving-on.html
    fn mix64(mut v: u64) -> u64 {
        v = (v ^ (v >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        v = (v ^ (v >> 27)).wrapping_mul(0x94d049bb133111eb);
        v ^ (v >> 31)
    }

    // Returns the gamma value to use for a new split instance. Uses the 64bit
    // mix function from MurmurHash3:
    // - https://github.com/aappleby/smhasher/wiki/MurmurHash3
    fn mixg(mut v: u64) -> u64 {
        v = (v ^ (v >> 33)).wrapping_mul(0xff51afd7ed558ccd);
        v = (v ^ (v >> 33)).wrapping_mul(0xc4ceb9fe1a85ec53);
        v = (v ^ (v >> 33)) | 1;
        if (v ^ (v >> 1)).count_ones() < 24 {
            v ^ 0xaaaaaaaaaaaaaaaa
        } else {
            v
        }
    }

    /// Create a new instance with the given seed.
    ///
    /// The seed is used unmodified as the internal state of the SplitMix64
    /// RNG, and thus will produce the same results as other SplitMix64
    /// implementations with this seed.
    pub fn with_seed(seed: u64) -> Self {
        Self {
            state: seed,
        }
    }

    fn step(&mut self, gamma: u64) -> u64 {
        self.state = self.state.wrapping_add(gamma);
        self.state
    }

    /// Produce the next 32-bit random number.
    pub fn next32(&mut self) -> u32 {
        Mix64::mix32(self.step(Self::GAMMA))
    }

    /// Produce the next 64-bit random number.
    pub fn next64(&mut self) -> u64 {
        Mix64::mix64(self.step(Self::GAMMA))
    }

    /// Split this random number generator in two.
    ///
    /// Works like [`SplitMix64::split()`].
    pub fn split(&mut self) -> SplitMix64 {
        SplitMix64::with(
            Mix64::mix64(self.step(Self::GAMMA)),
            Mix64::mixg(self.step(Self::GAMMA)),
        )
    }
}

impl SplitMix64 {
    fn with(seed: u64, gamma: u64) -> Self {
        Self {
            mix: Mix64::with_seed(seed),
            gamma: gamma,
        }
    }

    /// Create a new instance with the given seed.
    ///
    /// The seed is used unmodified as the internal state of the Mix64
    /// RNG, and thus will produce the same results as other Mix64
    /// implementations with this seed.
    pub fn with_seed(seed: u64) -> Self {
        Self::with(seed, Mix64::GAMMA)
    }

    /// Create a new instance from an unsplit `Mix64`.
    pub fn from_mix64(v: Mix64) -> Self {
        Self::with(v.state, Mix64::GAMMA)
    }

    /// Produce the next 32-bit random number.
    pub fn next32(&mut self) -> u32 {
        Mix64::mix32(self.mix.step(self.gamma))
    }

    /// Produce the next 64-bit random number.
    pub fn next64(&mut self) -> u64 {
        Mix64::mix64(self.mix.step(self.gamma))
    }

    /// Split this random number generator in two.
    ///
    /// This splits the RNG in two. The RNGs share no state. However, with very
    /// high probability, the set of values collectively generated by the two
    /// RNGs has the same statistical properties as if the same quantity of
    /// values were generated by a single RNG.  Either or both of the two RNGs
    /// may be further split and the same expected statistical properties apply
    /// to the entire set of generators constructed by such recursive
    /// splitting.
    ///
    /// The state of the original RNG is the same as if it was advanced twice.
    pub fn split(&mut self) -> Self {
        Self::with(
            Mix64::mix64(self.mix.step(self.gamma)),
            Mix64::mixg(self.mix.step(self.gamma)),
        )
    }
}

impl From<Mix64> for SplitMix64 {
    fn from(v: Mix64) -> Self {
        Self::from_mix64(v)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // Run some known-value-tests on Mix64 and SplitMix64.
    #[test]
    fn sm64_known() {
        {
            let mut rng0 = Mix64::with_seed(0);
            let mut rng1 = SplitMix64::with_seed(0);

            assert_eq!(rng0.next64(), 16294208416658607535);
            assert_eq!(rng1.next64(), 16294208416658607535);
            assert_eq!(rng0.next64(), 7960286522194355700);
            assert_eq!(rng1.next64(), 7960286522194355700);
            assert_eq!(rng0.next64(), 487617019471545679);
            assert_eq!(rng1.next64(), 487617019471545679);
            assert_eq!(rng0.next64(), 17909611376780542444);
            assert_eq!(rng1.next64(), 17909611376780542444);
        }

        {
            let mut rng0 = Mix64::with_seed(1234567);
            let mut rng1 = SplitMix64::with_seed(1234567);

            assert_eq!(rng0.next64(), 6457827717110365317);
            assert_eq!(rng1.next64(), 6457827717110365317);
            assert_eq!(rng0.next64(), 3203168211198807973);
            assert_eq!(rng1.next64(), 3203168211198807973);
            assert_eq!(rng0.next64(), 9817491932198370423);
            assert_eq!(rng1.next64(), 9817491932198370423);
            assert_eq!(rng0.next64(), 4593380528125082431);
            assert_eq!(rng1.next64(), 4593380528125082431);
        }
    }

    // Verify splitting behavior.
    #[test]
    fn sm64_split() {
        // Verify splitting the RNG advances it twice.
        {
            let mut rng0 = Mix64::with_seed(0);
            let mut rng1 = Mix64::with_seed(0);

            assert_eq!(rng0.next64(), rng1.next64());
            let _ = rng0.split();
            let _ = rng1.next64();
            let _ = rng1.next64();
            assert_eq!(rng0.next64(), rng1.next64());
        }

        // Verify splitting a `Mix64` produces the same as splitting a
        // `SplitMix64`.
        {
            let mut rng0 = Mix64::with_seed(0);
            let mut rng1 = SplitMix64::with_seed(0);

            assert_eq!(SplitMix64::from_mix64(rng0), rng1);
            assert_eq!(rng0.split(), rng1.split());
        }
    }
}
