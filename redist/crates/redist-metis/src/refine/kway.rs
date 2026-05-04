use crate::graph::{CsrGraph, Partition};
use crate::refine::Refiner;
use super::fm::FiducciaMattheyses;

pub struct GreedyKWay { pub niter: u32 }

impl Refiner for GreedyKWay {
    fn refine(&self, g: &CsrGraph, p: Partition) -> Partition {
        FiducciaMattheyses { niter: self.niter, ..FiducciaMattheyses::default() }.refine(g, p)
    }
}
