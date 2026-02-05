pub use rand::Rng;

pub trait Re<T> {
    type Item;
    fn re(&self, t: &T) -> impl Iterator<Item = Self::Item>;
}

mod shuffle;
mod bootstrap;
mod jackknife;
mod subsampling;
mod block_bootstrap;
mod wild_bootstrap;

pub use bootstrap::Bootstrap;
pub use jackknife::Jackknife;
pub use shuffle::Shuffle;
pub use subsampling::{Subsample, SamplingMode};
