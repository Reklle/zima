use rand::Rng;
use crate::Sample;
use super::Re;

#[derive(Clone, Copy, Default)]
pub struct Bootstrap<R: Rng> {
    pub rng: R,
}

impl<R: Rng> Bootstrap<R> {
    pub fn new(rng: R) -> Self {
        Self { rng }
    }
}

impl<T: Copy, R: Rng + Clone> Re<Sample<T>> for Bootstrap<R> {
    type Item = Sample<T>;

    fn re(&self, sample: &Sample<T>) -> impl Iterator<Item = Self::Item> {
        Box::new(BootstrapIter::new(&sample.data, self.rng.clone()))
    }
}

pub struct BootstrapIter<'a, T, R: Rng> {
    data: &'a [T],
    rng: R,
    buffer: Vec<T>,
}

impl<'a, T: Copy, R: Rng> BootstrapIter<'a, T, R> {
    fn new(data: &'a [T], rng: R) -> Self {
        Self {
            buffer: Vec::with_capacity(data.len()),
            data,
            rng,
        }
    }
}

impl<'a, T: Copy, R: Rng> Iterator for BootstrapIter<'a, T, R> {
    type Item = Sample<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let n = self.data.len();
        self.buffer.clear();
        self.buffer.reserve_exact(n);

        unsafe {
            self.buffer.set_len(n);
            for i in 0..n {
                let idx = self.rng.gen_range(0..n);
                *self.buffer.get_unchecked_mut(i) = *self.data.get_unchecked(idx);
            }
        }

        Some(Sample::new(std::mem::take(&mut self.buffer)))
    }
}
