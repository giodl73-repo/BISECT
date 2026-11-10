//! Fixed-width rand 0.8.6 SmallRng for 32-bit native sampler replay.
//! This is a non-cryptographic generator; it does not supply entropy.
// Xoshiro128++ and byte filling adapted from Rand 0.8.6 / rand_core 0.6.4.
// Copyright 2018 Developers of the Rand project; Copyright 2014 Rust Project
// Developers. Distributed under MIT; see ../licenses/rand-0.8.6-LICENSE-MIT.
use rand::{Error, RngCore, SeedableRng};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeSmallRng32 {
    state: [u32; 4],
}

impl SeedableRng for NativeSmallRng32 {
    type Seed = [u8; 16];

    fn from_seed(mut seed: Self::Seed) -> Self {
        if seed.iter().all(|&byte| byte == 0) {
            // The underlying Xoshiro128++ uses SplitMix64 only for its
            // all-zero raw seed. The SmallRng wrapper's seed_from_u64 uses
            // the default rand_core expansion instead.
            let mut splitmix = 0u64;
            for chunk in seed.chunks_exact_mut(8) {
                splitmix = splitmix.wrapping_add(0x9e3779b97f4a7c15);
                let mut z = splitmix;
                z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
                z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
                chunk.copy_from_slice(&(z ^ (z >> 31)).to_le_bytes());
            }
        }
        let mut state = [0; 4];
        for (word, bytes) in state.iter_mut().zip(seed.chunks_exact(4)) {
            *word = u32::from_le_bytes(bytes.try_into().expect("four-byte seed word"));
        }
        Self { state }
    }
}

impl RngCore for NativeSmallRng32 {
    fn next_u32(&mut self) -> u32 {
        let result = self.state[0]
            .wrapping_add(self.state[3])
            .rotate_left(7)
            .wrapping_add(self.state[0]);
        let t = self.state[1] << 9;
        self.state[2] ^= self.state[0];
        self.state[3] ^= self.state[1];
        self.state[1] ^= self.state[2];
        self.state[0] ^= self.state[3];
        self.state[2] ^= t;
        self.state[3] = self.state[3].rotate_left(11);
        result
    }

    fn next_u64(&mut self) -> u64 {
        let low = u64::from(self.next_u32());
        let high = u64::from(self.next_u32());
        low | (high << 32)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        let mut words = dest.chunks_exact_mut(8);
        for word in &mut words {
            word.copy_from_slice(&self.next_u64().to_le_bytes());
        }
        let rest = words.into_remainder();
        match rest.len() {
            0 => {}
            len @ 1..=4 => rest.copy_from_slice(&self.next_u32().to_le_bytes()[..len]),
            len => rest.copy_from_slice(&self.next_u64().to_le_bytes()[..len]),
        }
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}
