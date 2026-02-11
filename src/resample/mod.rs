pub use rand::Rng;

pub trait Re<T> {
    type Item;

    /// Sequential resampling
    fn re(&self, t: &T) -> impl Iterator<Item = Self::Item>;
}

#[cfg(feature = "rayon")]
pub trait RePar<T>: Re<T> {
    type Item;

    /// Parallel resampling using `rayon`
    fn re_par(&self, t: &T) -> impl rayon::prelude::ParallelIterator<Item = Self::Item>;
}


pub trait ReStrategy<T>
{
    #[inline(always)]
    fn mutate(
        &mut self,
        data: &T,
        output: &mut T,
    );
}

/// Rng in strategy
struct ReIter<S, T> {
    strategy: S,
    data: T,
    buffer: T,
}

impl<S, T> Iterator for ReIter<S, T>
where
    T: Copy,
    S: ReStrategy<T>
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.strategy.mutate(&self.data, &mut self.buffer);
        Some(self.buffer)
    }
}

struct Bootstrap<R> {
    rng: R,
}

impl<T, R> ReStrategy<Vec<T>> for Bootstrap<R>
where
    R: Rng,
    T: Copy,
{
    fn mutate(&mut self, data: &Vec<T>, buffer: &mut Vec<T>) {
        let n = data.len();
        for i in 0..n {
           	let idx = self.rng.gen_range(0..n);
           	unsafe {
                *buffer.get_unchecked_mut(i) = *data.get_unchecked(idx);
            }
        }
    }
}

mod shuffle;
mod bootstrap;
mod jackknife;
mod subsampling;
mod flipper;
mod block_bootstrap;
mod wild_bootstrap;

pub use bootstrap::Bootstrap;
pub use jackknife::Jackknife;
pub use shuffle::Shuffle;
pub use subsampling::{Subsample, SamplingMode};
pub use flipper::*;
