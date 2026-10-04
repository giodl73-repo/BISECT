//! Serial counterparts to the iterator calls used by the browser engine.
//! Native builds continue to import Rayon instead.
pub trait IntoParallelIterator: IntoIterator + Sized {
    fn into_par_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}
impl<T: IntoIterator> IntoParallelIterator for T {}
