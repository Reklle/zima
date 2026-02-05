use rand::Rng;
use crate::Sample;
use super::Re;

/// Subsampling strategy selector
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SamplingMode {
    /// Sample without replacement (default)
    WithoutReplacement,
    /// Sample with replacement (bootstrap-style)
    WithReplacement,
}

#[derive(Clone)]
pub struct Subsample<R: Rng, F = fn(usize) -> usize>
where
    F: Fn(usize) -> usize + Clone,
{
    pub rng: R,
    /// Size policy: function mapping total size → subsample size
    /// Default: √n (square root subsampling)
    pub policy: F,
    /// Sampling strategy (default: without replacement)
    pub mode: SamplingMode,
}

impl<R: Rng> Subsample<R, fn(usize) -> usize> {
    pub fn new(rng: R) -> Self {
        Self {
            rng,
            policy: |n| (n as f64).sqrt() as usize,
            mode: SamplingMode::WithoutReplacement,
        }
    }
}

impl<R: Rng, F> Subsample<R, F>
where
    F: Fn(usize) -> usize + Clone,
{
    pub fn with_policy(rng: R, policy: F) -> Self {
        Self {
            rng,
            policy,
            mode: SamplingMode::WithoutReplacement,
        }
    }

    pub fn with_mode(mut self, mode: SamplingMode) -> Self {
        self.mode = mode;
        self
    }
}

impl<T: Copy, R: Rng + Clone, F> Re<Sample<T>> for Subsample<R, F>
where
    F: Fn(usize) -> usize + Clone,
{
    type Item = Sample<T>;

    fn re(&self, sample: &Sample<T>) -> impl Iterator<Item = Self::Item> {
        let subsample_size = (self.policy)(sample.data.len()).min(sample.data.len());
        Box::new(SubsampleIter::new(
            &sample.data,
            self.rng.clone(),
            subsample_size,
            self.mode,
        ))
    }
}

pub struct SubsampleIter<'a, T, R: Rng> {
    data: &'a [T],
    rng: R,
    buffer: Vec<T>,
    subsample_size: usize,
    mode: SamplingMode,
    // Cache for reservoir sampling state
    reservoir_idx: usize,
}

impl<'a, T: Copy, R: Rng> SubsampleIter<'a, T, R> {
    fn new(data: &'a [T], rng: R, subsample_size: usize, mode: SamplingMode) -> Self {
        Self {
            buffer: Vec::with_capacity(subsample_size),
            data,
            rng,
            subsample_size,
            mode,
            reservoir_idx: 0,
        }
    }

    #[inline(always)]
    fn sample_without_replacement(&mut self) -> Sample<T> {
        let n = self.data.len();
        let k = self.subsample_size;

        // Adaptive algorithm selection:
        // - Small k (< 25% of n): Reservoir sampling (O(n) time, O(k) space)
        // - Large k (≥ 25% of n): Partial Fisher-Yates (O(k) time, reuses full buffer)
        if k == 0 {
            return Sample::new(Vec::new());
        }

        if k < n / 4 {
            // RESERVOIR SAMPLING (optimal for small k)
            self.buffer.clear();
            self.buffer.reserve_exact(k);

            // Phase 1: Fill reservoir with first k elements
            unsafe {
                self.buffer.set_len(k);
                std::ptr::copy_nonoverlapping(
                    self.data.as_ptr(),
                    self.buffer.as_mut_ptr(),
                    k.min(n),
                );
            }

            // Phase 2: Probabilistic replacement for remaining elements
            let mut i = k;
            while i < n {
                // Generate random index in [0, i] inclusive
                let j = self.rng.gen_range(0..=i);
                if j < k {
                    unsafe {
                        *self.buffer.get_unchecked_mut(j) = *self.data.get_unchecked(i);
                    }
                }
                i += 1;
            }

            Sample::new(std::mem::take(&mut self.buffer))
        } else {
            // PARTIAL FISHER-YATES (optimal for large k)
            // Reuse full buffer to avoid reallocations
            if self.buffer.capacity() < n {
                self.buffer.reserve_exact(n);
            }

            unsafe {
                self.buffer.set_len(n);
                std::ptr::copy_nonoverlapping(
                    self.data.as_ptr(),
                    self.buffer.as_mut_ptr(),
                    n,
                );

                // Shuffle only first k elements
                let ptr = self.buffer.as_mut_ptr();
                for i in (n - k..n).rev() {
                    let j = self.rng.gen_range(0..=i);
                    let tmp = *ptr.add(i);
                    *ptr.add(i) = *ptr.add(j);
                    *ptr.add(j) = tmp;
                }

                // Truncate to k elements (no allocation)
                self.buffer.set_len(k);
            }

            Sample::new(std::mem::take(&mut self.buffer))
        }
    }

    #[inline(always)]
    fn sample_with_replacement(&mut self) -> Sample<T> {
        let n = self.data.len();
        let k = self.subsample_size;

        self.buffer.clear();
        self.buffer.reserve_exact(k);

        unsafe {
            self.buffer.set_len(k);
            let out_ptr = self.buffer.as_mut_ptr();
            let data_ptr = self.data.as_ptr();

            // Unrolled loop for better ILP (Instruction-Level Parallelism)
            let mut i = 0;
            while i + 3 < k {
                let idx0 = self.rng.gen_range(0..n);
                let idx1 = self.rng.gen_range(0..n);
                let idx2 = self.rng.gen_range(0..n);
                let idx3 = self.rng.gen_range(0..n);

                *out_ptr.add(i) = *data_ptr.add(idx0);
                *out_ptr.add(i + 1) = *data_ptr.add(idx1);
                *out_ptr.add(i + 2) = *data_ptr.add(idx2);
                *out_ptr.add(i + 3) = *data_ptr.add(idx3);
                i += 4;
            }

            // Handle remaining elements
            while i < k {
                let idx = self.rng.gen_range(0..n);
                *out_ptr.add(i) = *data_ptr.add(idx);
                i += 1;
            }
        }

        Sample::new(std::mem::take(&mut self.buffer))
    }
}

impl<'a, T: Copy, R: Rng> Iterator for SubsampleIter<'a, T, R> {
    type Item = Sample<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.data.is_empty() || self.subsample_size == 0 {
            return Some(Sample::new(Vec::new()));
        }

        Some(match self.mode {
            SamplingMode::WithoutReplacement => self.sample_without_replacement(),
            SamplingMode::WithReplacement => self.sample_with_replacement(),
        })
    }
}

// Common policy functions (zero-cost abstractions)
pub fn sqrt_policy(n: usize) -> usize {
    (n as f64).sqrt() as usize
}

pub fn log_policy(n: usize) -> usize {
    (n as f64).ln().max(1.0) as usize
}

pub fn fixed_ratio_policy(ratio: f64) -> impl Fn(usize) -> usize {
    move |n| ((n as f64) * ratio).max(1.0) as usize
}

pub fn fixed_size_policy(size: usize) -> impl Fn(usize) -> usize {
    move |_| size
}

// impl Subsample {
//     /// Default policy: √n rule (m = ⌊√n⌋ ∨ 1)
//     ///
//     /// Statistically optimal for:
//     /// - Distribution function estimation
//     /// - Variance stabilization in bootstrap
//     /// - Minimizing MSE for smooth functionals
//     pub fn default() -> Self {
//         Self::sqrt()
//     }

//     /// Power law policy: m = ⌊n^α⌋ ∨ 1 where α ∈ (0, 1]
//     ///
//     /// # Statistical guidance:
//     /// - α = 0.5: √n rule (default)
//     /// - α = 0.67: m-out-of-n bootstrap for irregular parameters
//     /// - α = 1.0: equivalent to standard bootstrap (not recommended for subsampling)
//     ///
//     /// # Panics
//     /// Panics if α ≤ 0.0 or α > 1.0
//     pub fn power(alpha: f64) -> Self {
//         assert!(alpha > 0.0 && alpha <= 1.0, "Power law exponent α must satisfy 0 < α ≤ 1");
//         Self {
//             policy: move |n| (n as f64).powf(alpha) as usize | 1,
//         }
//     }

//     /// Fraction policy: m = ⌊f·n⌋ ∨ 1 where f ∈ (0, 1]
//     ///
//     /// # Panics
//     /// Panics if f ≤ 0.0 or f > 1.0
//     pub fn fraction(f: f64) -> Self {
//         assert!(f > 0.0 && f <= 1.0, "Fraction must satisfy 0 < f ≤ 1");
//         Self {
//             policy: move |n| ((n as f64) * f) as usize | 1,
//         }
//     }

//     /// Fixed size policy (use with caution for small samples)
//     ///
//     /// # Panics
//     /// Panics if m == 0
//     pub fn fixed(m: usize) -> Self {
//         assert!(m > 0, "Fixed subsample size must be positive");
//         Self {
//             policy: move |_| m,
//         }
//     }

//     /// √n rule: m = ⌊√n⌋ ∨ 1 (statistically optimal default)
//     pub fn sqrt() -> Self {
//         Self {
//             policy: |n| (n as f64).sqrt() as usize | 1,
//         }
//     }
// }


// impl<'a, T: Copy, R: Rng> FusedIterator for SubsampleIter<'a, T, R> {}

// // Blanket implementation for common callable types
// impl<F> Clone for Subsample<F>
// where
//     F: Fn(usize) -> usize + Copy,
// {
//     fn clone(&self) -> Self {
//         Self {
//             policy: self.policy,
//         }
//     }
// }
