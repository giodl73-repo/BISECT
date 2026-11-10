//! Fixed-width rand-0.8.6 SmallRng compatibility for 64-bit native replay.
//! This is a non-cryptographic sampler RNG, not an entropy or security API.
// Xoshiro256++ and byte filling adapted from Rand 0.8.6 / rand_core 0.6.4.
// Copyright 2018 Developers of the Rand project; Copyright 2014 Rust Project
// Developers. Distributed under MIT; see ../licenses/rand-0.8.6-LICENSE-MIT.
use rand::{RngCore,SeedableRng,Error};

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct NativeSmallRng64{state:[u64;4]}
impl SeedableRng for NativeSmallRng64{
 type Seed=[u8;32];
 fn from_seed(seed:Self::Seed)->Self{
  let mut state=[0;4];
  if seed.iter().all(|&b|b==0){
   // The inner xoshiro all-zero fallback is SplitMix64, unlike SmallRng's
   // default SeedableRng::seed_from_u64 expansion (PCG32 in rand_core).
   let mut value=0u64;
   for item in &mut state{value=value.wrapping_add(0x9e3779b97f4a7c15);let mut z=value;z=(z^(z>>30)).wrapping_mul(0xbf58476d1ce4e5b9);z=(z^(z>>27)).wrapping_mul(0x94d049bb133111eb);*item=z^(z>>31);}
  }else{for (item,bytes) in state.iter_mut().zip(seed.chunks_exact(8)){*item=u64::from_le_bytes(bytes.try_into().unwrap());}}
  Self{state}
 }
 // Deliberately retain rand_core's default seed_from_u64, just as SmallRng
 // does. Calling xoshiro's SplitMix seed_from_u64 here changes native streams.
}
impl RngCore for NativeSmallRng64{
 fn next_u32(&mut self)->u32{(self.next_u64()>>32) as u32}
 fn next_u64(&mut self)->u64{
  let result=self.state[0].wrapping_add(self.state[3]).rotate_left(23).wrapping_add(self.state[0]);let t=self.state[1]<<17;
  self.state[2]^=self.state[0];self.state[3]^=self.state[1];self.state[1]^=self.state[2];self.state[0]^=self.state[3];self.state[2]^=t;self.state[3]=self.state[3].rotate_left(45);result
 }
 fn fill_bytes(&mut self,dest:&mut[u8]){
  let mut chunks=dest.chunks_exact_mut(8);for chunk in &mut chunks{chunk.copy_from_slice(&self.next_u64().to_le_bytes());}let rest=chunks.into_remainder();let n=rest.len();if n>4{rest.copy_from_slice(&self.next_u64().to_le_bytes()[..n]);}else if n>0{rest.copy_from_slice(&self.next_u32().to_le_bytes()[..n]);}
 }
 fn try_fill_bytes(&mut self,dest:&mut[u8])->Result<(),Error>{self.fill_bytes(dest);Ok(())}
}
