use crate::Sample;
use std::iter::FusedIterator;
use super::*;

#[derive(Clone, Copy, Default)]
pub struct Jackknife;

impl Jackknife {
    pub fn new() -> Self {
        Self
    }
}

impl<T: Copy> Re<Sample<T>> for Jackknife {
    type Item = Sample<T>;
    fn re(&self, sample: &Sample<T>) -> impl Iterator<Item = Self::Item> {
        JackknifeIter::new(&sample.data)
    }
}

pub struct JackknifeIter<'a, T: Copy> {
    data: &'a [T],
    buffer: Vec<T>,
    omit_idx: usize,
    n: usize,
}

impl<'a, T: Copy> JackknifeIter<'a, T> {
    #[inline(always)]
    fn new(data: &'a [T]) -> Self {
        let n = data.len();
        let mut buffer = Vec::with_capacity(n.saturating_sub(1));
        // Safety: buffer is empty, set_len will be called before reads
        unsafe { buffer.set_len(0) };

        Self {
            data,
            buffer,
            omit_idx: 0,
            n,
        }
    }
}

impl<'a, T: Copy> Iterator for JackknifeIter<'a, T> {
    type Item = Sample<T>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.omit_idx >= self.n {
            return None;
        }

        let omit = self.omit_idx;
        self.omit_idx += 1;
        let new_len = self.n - 1;

        // Reuse buffer capacity without reallocation
        if self.buffer.capacity() < new_len {
            self.buffer.reserve_exact(new_len);
        }

        unsafe {
            // Critical optimization: use single contiguous write operation
            // Compiler will autovectorize these copies for primitive types
            self.buffer.set_len(new_len);

            // Copy prefix [0..omit)
            if omit > 0 {
                std::ptr::copy_nonoverlapping(
                    self.data.as_ptr(),
                    self.buffer.as_mut_ptr(),
                    omit,
                );
            }

            // Copy suffix [omit+1..n) -> [omit..end)
            // Single branch-free operation when omit is in middle
            if omit < self.n - 1 {
                std::ptr::copy_nonoverlapping(
                    self.data.as_ptr().add(omit + 1),
                    self.buffer.as_mut_ptr().add(omit),
                    self.n - omit - 1,
                );
            }
        }

        // Zero-cost transfer: buffer capacity preserved for next iteration
        // mem::replace avoids double-free while maintaining capacity
        let sample_data = std::mem::replace(
            &mut self.buffer,
            Vec::with_capacity(new_len),
        );

        Some(Sample::new(sample_data))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.n.saturating_sub(self.omit_idx);
        (remaining, Some(remaining))
    }
}

impl<'a, T: Copy> ExactSizeIterator for JackknifeIter<'a, T> {}
impl<'a, T: Copy> FusedIterator for JackknifeIter<'a, T> {}
