use rand::Rng;
use crate::Sample;
use super::Re;

#[derive(Clone, Copy, Default)]
pub struct Shuffle<R: Rng> {
    pub rng: R,
}

impl<R: Rng> Shuffle<R> {
    pub fn new(rng: R) -> Self {
        Self { rng }
    }
}

impl<T: Copy, R: Rng + Clone> Re<Sample<T>> for Shuffle<R> {
    type Item = Sample<T>;

    fn re(&self, sample: &Sample<T>) -> impl Iterator<Item = Self::Item> {
        Box::new(ShuffleIter::new(&sample.data, self.rng.clone()))
    }
}

pub struct ShuffleIter<'a, T, R: Rng> {
    data: &'a [T],
    rng: R,
    buffer: Vec<T>,
}

impl<'a, T: Copy, R: Rng> ShuffleIter<'a, T, R> {
    fn new(data: &'a [T], rng: R) -> Self {
        // Pre-allocate buffer with exact capacity to avoid reallocations
        Self {
            buffer: Vec::with_capacity(data.len()),
            data,
            rng,
        }
    }
}

impl<'a, T: Copy, R: Rng> Iterator for ShuffleIter<'a, T, R> {
    type Item = Sample<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let n = self.data.len();

        // Reuse buffer allocation with zero-copy semantics
        unsafe {
            // Ensure capacity (defensive check; should never trigger with fixed-size input)
            if self.buffer.capacity() < n {
                self.buffer.reserve_exact(n);
            }
            // Set length and perform raw memcpy (bypasses bounds checks)
            self.buffer.set_len(n);
            std::ptr::copy_nonoverlapping(self.data.as_ptr(), self.buffer.as_mut_ptr(), n);
        }

        // Optimized Fisher-Yates shuffle (in-place, no allocations)
        if n > 1 {
            let ptr = self.buffer.as_mut_ptr();
            // Iterate from last index down to 1 (avoids unnecessary swap at index 0)
            for i in (1..n).rev() {
                // Inclusive range [0, i] - critical for uniform distribution
                let j = self.rng.gen_range(0..=i);
                // Unchecked pointer swaps (bypasses bounds checks)
                unsafe {
                    let tmp = *ptr.add(i);
                    *ptr.add(i) = *ptr.add(j);
                    *ptr.add(j) = tmp;
                }
            }
        }

        // Transfer ownership to Sample while preserving buffer capacity
        Some(Sample::new(std::mem::take(&mut self.buffer)))
    }
}
