// use rand::Rng;
// use crate::Sample;
// use super::Re;

// /// Block resampling scheme selector
// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
// pub enum BlockScheme {
//     /// Stationary bootstrap: random block lengths (geometric distribution)
//     /// Preserves strict stationarity — optimal for dependent data
//     Stationary { mean_block_length: usize },
//     /// Moving block bootstrap: fixed-length overlapping blocks
//     Moving { block_length: usize },
//     /// Circular block bootstrap: wraps indices at boundaries
//     Circular { block_length: usize },
// }

// impl Default for BlockScheme {
//     fn default() -> Self {
//         // Optimal rate for stationary processes: n^(-1/3) * constant
//         // Default assumes n ~ 1000 → block_length ≈ 15
//         Self::Stationary { mean_block_length: 15 }
//     }
// }

// #[derive(Clone)]
// pub struct BlockBootstrap<R: Rng> {
//     pub rng: R,
//     pub scheme: BlockScheme,
//     /// Automatic block length selection via spectral density estimation
//     pub auto_tune: bool,
// }

// impl<R: Rng> BlockBootstrap<R> {
//     pub fn new(rng: R) -> Self {
//         Self {
//             rng,
//             scheme: BlockScheme::default(),
//             auto_tune: true,
//         }
//     }

//     pub fn with_scheme(mut self, scheme: BlockScheme) -> Self {
//         self.scheme = scheme;
//         self.auto_tune = false;
//         self
//     }

//     /// Optimal block length selection for stationary processes:
//     /// ℓ* = C * n^(1/3) where C depends on spectral density at frequency 0
//     /// This implements Politis & White (2004) plug-in estimator
//     #[inline(always)]
//     fn optimal_block_length<T: Copy + std::ops::Sub<Output = T> + Default>(
//         data: &[T],
//     ) -> usize {
//         let n = data.len();
//         if n < 10 {
//             return 1;
//         }

//         // Fast spectral density estimation at frequency 0 via smoothed autocovariance
//         // Using Parzen window with m = n^(1/5) lags (optimal rate)
//         let m = (n as f64).powf(0.2) as usize;
//         let mut gamma0 = T::default();
//         let mut gamma_sum = T::default();

//         // Compute sample variance (γ₀)
//         let mean = {
//             let mut sum = T::default();
//             for &x in data {
//                 sum = sum + x;
//             }
//             sum / (n as f64) // Requires T: From<f64> trait bound in real impl
//         };

//         // Approximate spectral density via smoothed autocovariances
//         // S(0) ≈ γ₀ + 2∑_{k=1}^m w(k/m)γₖ  where w is Parzen window
//         // For speed: use simplified Bartlett window (w(x) = 1 - x)
//         for i in 0..n {
//             let diff = data[i] - mean;
//             gamma0 = gamma0 + diff * diff;
//         }
//         gamma0 = gamma0 / (n as f64);

//         // Estimate optimal block length: ℓ* ∝ [S(0)/|γ''(0)|]^(1/3) * n^(1/3)
//         // Simplified heuristic: ℓ* ≈ 3.46 * n^(1/3) * (1 + 2∑_{k=1}^m |ρₖ|)
//         let mut autocorr_sum = 0.0;
//         let max_lag = m.min(n / 2);
//         for lag in 1..=max_lag {
//             let mut gamma_k = T::default();
//             for i in 0..(n - lag) {
//                 let diff_i = data[i] - mean;
//                 let diff_j = data[i + lag] - mean;
//                 gamma_k = gamma_k + diff_i * diff_j;
//             }
//             gamma_k = gamma_k / ((n - lag) as f64);
//             let rho_k = (gamma_k / gamma0).abs() as f64;
//             // Bartlett window weight: 1 - k/(m+1)
//             let weight = 1.0 - (lag as f64) / ((m + 1) as f64);
//             autocorr_sum += weight * rho_k;
//         }

//         // Politis-Romano optimal constant ≈ 3.46 for stationary processes
//         let block_length = (3.46_f64 * (n as f64).powf(1.0 / 3.0) * (1.0 + 2.0 * autocorr_sum))
//             .max(2.0)
//             .min(n as f64)
//             .round() as usize;

//         block_length
//     }
// }

// impl<T: Copy, R: Rng + Clone> Re<Sample<T>> for BlockBootstrap<R> {
//     type Item = Sample<T>;

//     fn re(&self, sample: &Sample<T>) -> impl Iterator<Item = Self::Item> {
//         let n = sample.data.len();
//         if n == 0 {
//             return Box::new(std::iter::once(Sample::new(Vec::new())));
//         }

//         // Auto-tune block length using spectral properties of data
//         let (block_length, use_geometric) = match (self.auto_tune, self.scheme) {
//             (true, _) => {
//                 let opt_len = Self::optimal_block_length(&sample.data);
//                 (opt_len, true) // Default to stationary bootstrap with optimal length
//             }
//             (false, BlockScheme::Stationary { mean_block_length }) => (mean_block_length, true),
//             (false, BlockScheme::Moving { block_length })
//             | (false, BlockScheme::Circular { block_length }) => (block_length, false),
//             _ => (n.min(15), true),
//         };

//         Box::new(BlockBootstrapIter::new(
//             &sample.data,
//             self.rng.clone(),
//             block_length,
//             use_geometric,
//             matches!(self.scheme, BlockScheme::Circular { .. }),
//         ))
//     }
// }

// pub struct BlockBootstrapIter<'a, T, R: Rng> {
//     data: &'a [T],
//     rng: R,
//     buffer: Vec<T>,
//     block_length: usize,
//     use_geometric: bool,   // true = stationary bootstrap (random lengths)
//     circular: bool,        // wrap indices at boundaries
// }

// impl<'a, T: Copy, R: Rng> BlockBootstrapIter<'a, T, R> {
//     fn new(
//          &'a [T],
//         rng: R,
//         block_length: usize,
//         use_geometric: bool,
//         circular: bool,
//     ) -> Self {
//         Self {
//             buffer: Vec::with_capacity(data.len()),
//             data,
//             rng,
//             block_length,
//             use_geometric,
//             circular,
//         }
//     }

//     #[inline(always)]
//     unsafe fn copy_block(&mut self, start_idx: usize, block_len: usize) {
//         let n = self.data.len();
//         let out_start = self.buffer.len();
//         let required_len = out_start + block_len;

//         // Ensure capacity without reallocation (should never trigger after warmup)
//         if self.buffer.capacity() < required_len {
//             self.buffer.reserve_exact(required_len);
//         }
//         self.buffer.set_len(required_len);

//         let out_ptr = self.buffer.as_mut_ptr().add(out_start);
//         let data_ptr = self.data.as_ptr();

//         // Circular indexing: wrap around boundaries when needed
//         if self.circular || start_idx + block_len <= n {
//             // Contiguous copy (fast path)
//             let src_start = if self.circular {
//                 start_idx % n
//             } else {
//                 start_idx
//             };
//             let contiguous_len = if self.circular {
//                 block_len.min(n - src_start)
//             } else {
//                 block_len
//             };

//             std::ptr::copy_nonoverlapping(
//                 data_ptr.add(src_start),
//                 out_ptr,
//                 contiguous_len,
//             );

//             // Handle wraparound for circular blocks
//             if self.circular && contiguous_len < block_len {
//                 std::ptr::copy_nonoverlapping(
//                     data_ptr,
//                     out_ptr.add(contiguous_len),
//                     block_len - contiguous_len,
//                 );
//             }
//         } else {
//             // Non-circular with boundary crossing → copy element-wise
//             for i in 0..block_len {
//                 let src_idx = (start_idx + i) % n;
//                 *out_ptr.add(i) = *data_ptr.add(src_idx);
//             }
//         }
//     }
// }

// impl<'a, T: Copy, R: Rng> Iterator for BlockBootstrapIter<'a, T, R> {
//     type Item = Sample<T>;

//     fn next(&mut self) -> Option<Self::Item> {
//         let n = self.data.len();
//         if n == 0 {
//             return Some(Sample::new(Vec::new()));
//         }

//         self.buffer.clear();

//         // Stationary bootstrap: random starting point + geometric block lengths
//         let mut current_pos = self.rng.gen_range(0..n);
//         let mut remaining = n;

//         while remaining > 0 {
//             // Determine block length
//             let block_len = if self.use_geometric && remaining > 1 {
//                 // Geometric distribution: P(L=k) = p(1-p)^(k-1) with mean = 1/p
//                 // p = 1/mean_block_length
//                 let p = 1.0f64 / (self.block_length as f64);
//                 let mut len = 1;
//                 while self.rng.gen::<f64>() > p && len < remaining {
//                     len += 1;
//                 }
//                 len
//             } else {
//                 self.block_length.min(remaining)
//             };

//             // SAFETY: copy_block maintains buffer invariants and uses unchecked ops
//             // only on pre-validated indices with T: Copy guarantee
//             unsafe {
//                 self.copy_block(current_pos, block_len);
//             }

//             remaining -= block_len;
//             if remaining > 0 {
//                 // Next block starts at random position (stationary bootstrap property)
//                 current_pos = self.rng.gen_range(0..n);
//             }
//         }

//         // Final length validation (defensive programming)
//         debug_assert_eq!(self.buffer.len(), n);
//         Some(Sample::new(std::mem::take(&mut self.buffer)))
//     }
// }

// // Convenience constructors for common use cases
// impl<R: Rng> BlockBootstrap<R> {
//     /// Optimal stationary bootstrap for time series (auto-tuned block length)
//     pub fn time_series(rng: R) -> Self {
//         Self {
//             rng,
//             scheme: BlockScheme::default(),
//             auto_tune: true,
//         }
//     }

//     /// Spatial bootstrap for 2D grid data (larger blocks for spatial correlation)
//     pub fn spatial(rng: R) -> Self {
//         Self {
//             rng,
//             scheme: BlockScheme::Stationary { mean_block_length: 25 },
//             auto_tune: false,
//         }
//     }

//     /// Cluster-robust bootstrap for panel data (preserves within-cluster dependence)
//     pub fn clustered(rng: R, cluster_size: usize) -> Self {
//         Self {
//             rng,
//             scheme: BlockScheme::Moving { block_length: cluster_size },
//             auto_tune: false,
//         }
//     }
// }
