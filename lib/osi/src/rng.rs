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

/// Random Number Generator using Xoshiro256++.
///
/// This is the state of the Xoshiro256++ pseudo random number generator. It
/// uses 256-bit of state and generates 64-bit random numbers. This PRNG is
/// **not** cryptgraphically secure, but is otherwise a good fit for nearly
/// all purposes.
///
/// This implements `Clone` and `Copy` for verbatim copies. Use
/// [`jump128()`](Self::jump128) to produce non-verbatim copies with better
/// random distribution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Xoshiro256pp {
    state: [u64; 4],
}

// Static calculation of the first few values from Mix64(0). The array is
// statically verified to contain no 0 values (which sufficiently resolves
// non-0 seeding requirements of some RNGs).
const MIX64_0: [u64; 4] = {
    let mut mix = Mix64::with_seed(0);
    let v = [mix.next64(); 4];

    let mut i = 0;
    while i < v.len() {
        assert!(v[i] != 0);
        i += 1;
    }

    v
};

impl Mix64 {
    const GAMMA: u64 = 0x9e3779b97f4a7c15;

    // Computes the 32 high bits of Stafford variant 4 mix64 function:
    // - http://zimbry.blogspot.com/2011/09/better-bit-mixing-improving-on.html
    const fn mix32(mut v: u64) -> u32 {
        v = (v ^ (v >> 33)).wrapping_mul(0x62a9d9ed799705f5);
        v = (v ^ (v >> 28)).wrapping_mul(0xcb24d0a5c88c35b3);
        (v >> 32) as u32
    }

    // Computes Stafford variant 13 mix64 function:
    // - http://zimbry.blogspot.com/2011/09/better-bit-mixing-improving-on.html
    const fn mix64(mut v: u64) -> u64 {
        v = (v ^ (v >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        v = (v ^ (v >> 27)).wrapping_mul(0x94d049bb133111eb);
        v ^ (v >> 31)
    }

    // Returns the gamma value to use for a new split instance. Uses the 64bit
    // mix function from MurmurHash3:
    // - https://github.com/aappleby/smhasher/wiki/MurmurHash3
    const fn mixg(mut v: u64) -> u64 {
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
    pub const fn with_seed(seed: u64) -> Self {
        Self {
            state: seed,
        }
    }

    const fn step(&mut self, gamma: u64) -> u64 {
        self.state = self.state.wrapping_add(gamma);
        self.state
    }

    /// Produce the next 32-bit random number.
    pub const fn next32(&mut self) -> u32 {
        Mix64::mix32(self.step(Self::GAMMA))
    }

    /// Produce the next 64-bit random number.
    pub const fn next64(&mut self) -> u64 {
        Mix64::mix64(self.step(Self::GAMMA))
    }

    /// Split this random number generator in two.
    ///
    /// Works like [`SplitMix64::split()`].
    pub const fn split(&mut self) -> SplitMix64 {
        SplitMix64::with(
            Mix64::mix64(self.step(Self::GAMMA)),
            Mix64::mixg(self.step(Self::GAMMA)),
        )
    }
}

impl SplitMix64 {
    const fn with(seed: u64, gamma: u64) -> Self {
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
    pub const fn with_seed(seed: u64) -> Self {
        Self::with(seed, Mix64::GAMMA)
    }

    /// Create a new instance from an unsplit `Mix64`.
    pub const fn from_mix64(v: Mix64) -> Self {
        Self::with(v.state, Mix64::GAMMA)
    }

    /// Produce the next 32-bit random number.
    pub const fn next32(&mut self) -> u32 {
        Mix64::mix32(self.mix.step(self.gamma))
    }

    /// Produce the next 64-bit random number.
    pub const fn next64(&mut self) -> u64 {
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
    pub const fn split(&mut self) -> Self {
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

impl Xoshiro256pp {
    const fn combine(s: &[u64; 4]) -> u64 {
        s[0].wrapping_add(s[3])
            .rotate_left(23)
            .wrapping_add(s[0])
    }

    const fn step(s: &mut [u64; 4]) {
        let t = s[1] << 17;

        s[2] ^= s[0];
        s[3] ^= s[1];
        s[1] ^= s[2];
        s[0] ^= s[3];

        s[2] ^= t;
        s[3] = s[3].rotate_left(45);
    }

    const fn jump(s: &mut [u64; 4], jump_table: &[u64; 4]) {
        let mut t = [0; 4];

        let mut i = 0;
        while i < 4 {
            let mut j = 0;
            while j < 64 {
                if (jump_table[i] & (1 << j)) != 0 {
                    t[0] ^= s[0];
                    t[1] ^= s[1];
                    t[2] ^= s[2];
                    t[3] ^= s[3];
                }
                Self::step(s);
                j += 1;
            }
            i += 1;
        }

        *s = t;
    }

    // Steps 2^128 times, using pre-calculated jump tables.
    const fn step128(s: &mut [u64; 4]) {
        Self::jump(
            s,
            &[
                0x180ec6d33cfd0aba,
                0xd5a61266f0c9392c,
                0xa9582618e03fc9aa,
                0x39abdc4529b1661c,
            ],
        )
    }

    // Steps 2^192 times, using pre-calculated jump tables.
    const fn step192(s: &mut [u64; 4]) {
        Self::jump(
            s,
            &[
                0x76e15d3efefdcbbf,
                0xc5004e441c522fb3,
                0x77710069854ee241,
                0x39109bb02acbe635,
            ],
        )
    }

    /// Create a new instance with the given seed.
    ///
    /// The seed is used unmodified as the internal state of the Xoshiro256++
    /// RNG, and thus will produce the same results as other implementations
    /// with this seed (except if the seed is 0, described below).
    ///
    /// A seed of all 0 is not allowed for Xoshiro256++. This implementation
    /// maps a seed of all 0 to `Self::from_splitmix64(0)`.
    pub const fn with_seed(seed: [u64; 4]) -> Self {
        if seed[0] == 0 && seed[1] == 0 && seed[2] == 0 && seed[3] == 0 {
            Self {
                state: MIX64_0,
            }
        } else {
            Self {
                state: seed,
            }
        }
    }

    /// Create a new instance from a SplitMix64.
    ///
    /// Use the [`SplitMix64`] instance to seed the 256-bit of state of a new
    /// RNG instance.
    pub const fn from_splitmix64(mix: &mut SplitMix64) -> Self {
        Self::with_seed([mix.next64(); 4])
    }

    /// Produce the next 64-bit random number.
    pub const fn next64(&mut self) -> u64 {
        let r = Self::combine(&self.state);
        Self::step(&mut self.state);
        r
    }

    /// Jump over the next 2^128 random numbers.
    pub const fn jump128(&mut self) {
        Self::step128(&mut self.state);
    }

    /// Jump over the next 2^192 random numbers.
    pub const fn jump192(&mut self) {
        Self::step192(&mut self.state);
    }
}

impl From<SplitMix64> for Xoshiro256pp {
    fn from(mut v: SplitMix64) -> Self {
        Self::from_splitmix64(&mut v)
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

    // Run some known-value-tests on Xoshiro256pp.
    #[test]
    fn xoshiro256pp_known() {
        {
            let mut rng0 = Xoshiro256pp::with_seed([1, 2, 3, 4]);

            assert_eq!(rng0.next64(), 41943041);
            assert_eq!(rng0.next64(), 58720359);
            assert_eq!(rng0.next64(), 3588806011781223);
            assert_eq!(rng0.next64(), 3591011842654386);

            rng0.jump128();

            assert_eq!(rng0.next64(), 10838999831620499216);
            assert_eq!(rng0.next64(), 8680420094678800874);
            assert_eq!(rng0.next64(), 9570055643283944810);
            assert_eq!(rng0.next64(), 7079802948504130534);

            rng0.jump192();

            assert_eq!(rng0.next64(), 7229965972965062926);
            assert_eq!(rng0.next64(), 2140690761664815708);
            assert_eq!(rng0.next64(), 5733913562642225265);
            assert_eq!(rng0.next64(), 10699737370828579003);
        }
    }
}
