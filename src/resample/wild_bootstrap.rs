// use rand::Rng;
// use crate::Sample;
// use super::Re;

// /// Weight distribution for wild bootstrap (critical for theoretical properties)
// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
// pub enum WeightDist {
//     /// Rademacher: P(w=±1) = 0.5 each
//     /// Simplest, good finite-sample properties
//     Rademacher,
//     /// Mammen (1993): P(w=(1-√5)/2)= (√5+1)/(2√5), P(w=(1+√5)/2)= (√5-1)/(2√5)
//     /// Optimal for smooth conditional distributions
//     Mammen,
//     /// Webb (2014): Six-point distribution with improved skewness/kurtosis matching
//     /// Best for highly skewed error distributions
//     Webb,
//     /// Standard normal: w ~ N(0,1)
//     /// Asymptotically equivalent but slower convergence
//     Normal,
//     /// Gamma(2,2) shifted: w = 2(γ-1) where γ~Gamma(2,2)
//     /// Good for positive-valued outcomes
//     Gamma,
// }

// impl Default for WeightDist {
//     fn default() -> Self {
//         Self::Mammen
//     }
// }

// /// Wild bootstrap configuration for heteroskedastic-robust inference
// #[derive(Clone)]
// pub struct WildBootstrap<R: Rng> {
//     pub rng: R,
//     pub weight_dist: WeightDist,
//     /// Center weights to ensure E[w]=0 (critical for bias correction)
//     pub center_weights: bool,
//     /// Scale weights to ensure Var[w]=1 (critical for variance consistency)
//     pub scale_weights: bool,
// }

// impl<R: Rng> WildBootstrap<R> {
//     pub fn new(rng: R) -> Self {
//         Self {
//             rng,
//             weight_dist: WeightDist::default(),
//             center_weights: true,
//             scale_weights: true,
//         }
//     }

//     pub fn with_distribution(mut self, dist: WeightDist) -> Self {
//         self.weight_dist = dist;
//         self
//     }

//     pub fn without_centering(mut self) -> Self {
//         self.center_weights = false;
//         self
//     }

//     pub fn without_scaling(mut self) -> Self {
//         self.scale_weights = false;
//         self
//     }
// }

// /// Core data structure: residuals + fitted values for reconstruction
// pub struct WildSample<T> {
//     pub residuals: Vec<T>,
//     pub fitted: Vec<T>,
//     pub weights: Option<Vec<T>>, // Precomputed weights for efficiency
// }

// impl<T: Copy> WildSample<T> {
//     pub fn new(residuals: Vec<T>, fitted: Vec<T>) -> Self {
//         assert_eq!(residuals.len(), fitted.len(), "Residuals and fitted must match");
//         Self {
//             residuals,
//             fitted,
//             weights: None,
//         }
//     }
// }

// impl<T: Copy, R: Rng + Clone> Re<WildSample<T>> for WildBootstrap<R> {
//     type Item = Sample<T>;

//     fn re(&self, wild_sample: &WildSample<T>) -> impl Iterator<Item = Self::Item> {
//         let n = wild_sample.residuals.len();
//         if n == 0 {
//             return Box::new(std::iter::once(Sample::new(Vec::new())));
//         }

//         // Precompute weights if scaling/centering needed (one-time cost)
//         let weights = if self.center_weights || self.scale_weights {
//             let mut weights = Vec::with_capacity(n);
//             let mut sum = T::default();
//             let mut sum_sq = T::default();

//             for _ in 0..n {
//                 let w = self.sample_weight();
//                 weights.push(w);
//                 sum = sum + w;
//                 sum_sq = sum_sq + w * w;
//             }

//             if self.center_weights {
//                 let mean = sum / (n as f64); // Requires T: From<f64>
//                 for w in &mut weights {
//                     *w = *w - mean;
//                 }
//             }

//             if self.scale_weights {
//                 let var = (sum_sq / (n as f64)) - (sum / (n as f64)) * (sum / (n as f64));
//                 let scale = (1.0f64 / var.sqrt()) as T; // Requires T: From<f64>
//                 for w in &mut weights {
//                     *w = *w * scale;
//                 }
//             }

//             Some(weights)
//         } else {
//             None
//         };

//         Box::new(WildBootstrapIter::new(
//             &wild_sample.residuals,
//             &wild_sample.fitted,
//             weights,
//             self.rng.clone(),
//             self.weight_dist,
//             self.center_weights,
//             self.scale_weights,
//         ))
//     }
// }

// pub struct WildBootstrapIter<'a, T, R: Rng> {
//     residuals: &'a [T],
//     fitted: &'a [T],
//     precomputed_weights: Option<Vec<T>>,
//     rng: R,
//     weight_dist: WeightDist,
//     buffer: Vec<T>,
//     center_weights: bool,
//     scale_weights: bool,
// }

// impl<'a, T: Copy, R: Rng> WildBootstrapIter<'a, T, R> {
//     fn new(
//         residuals: &'a [T],
//         fitted: &'a [T],
//         precomputed_weights: Option<Vec<T>>,
//         rng: R,
//         weight_dist: WeightDist,
//         center_weights: bool,
//         scale_weights: bool,
//     ) -> Self {
//         Self {
//             buffer: Vec::with_capacity(residuals.len()),
//             residuals,
//             fitted,
//             precomputed_weights,
//             rng,
//             weight_dist,
//             center_weights,
//             scale_weights,
//         }
//     }

//     #[inline(always)]
//     fn sample_weight(&mut self) -> T {
//         match self.weight_dist {
//             WeightDist::Rademacher => {
//                 if self.rng.gen::<bool>() {
//                     T::from(1.0)
//                 } else {
//                     T::from(-1.0)
//                 }
//             }
//             WeightDist::Mammen => {
//                 // Mammen two-point distribution: optimal theoretical properties
//                 // w1 = (1-√5)/2 ≈ -0.618, p1 = (√5+1)/(2√5) ≈ 0.7236
//                 // w2 = (1+√5)/2 ≈ 1.618, p2 = (√5-1)/(2√5) ≈ 0.2764
//                 if self.rng.gen::<f64>() < 0.72360679775 {
//                     T::from(-0.61803398875)
//                 } else {
//                     T::from(1.61803398875)
//                 }
//             }
//             WeightDist::Webb => {
//                 // Webb six-point distribution (2014): matches skewness & kurtosis
//                 // Points: ±√(3/2), ±√(1/2), 0 with probabilities 1/8, 3/8, 1/4
//                 let u = self.rng.gen::<f64>();
//                 if u < 0.125 {
//                     T::from(1.22474487139) // √(3/2)
//                 } else if u < 0.25 {
//                     T::from(-1.22474487139)
//                 } else if u < 0.625 {
//                     T::from(0.70710678118) // √(1/2)
//                 } else if u < 0.875 {
//                     T::from(-0.70710678118)
//                 } else {
//                     T::from(0.0)
//                 }
//             }
//             WeightDist::Normal => {
//                 T::from(self.rng.gen::<f64>() * 1.0) // Standard normal
//             }
//             WeightDist::Gamma => {
//                 // Gamma(2,2) shifted: w = 2(γ-1), E[w]=0, Var[w]=1
//                 let gamma = self.rng.gen::<f64>().gamma(2.0, 0.5); // shape=2, scale=0.5
//                 T::from(2.0 * (gamma - 1.0))
//             }
//         }
//     }
// }

// impl<'a, T: Copy, R: Rng> Iterator for WildBootstrapIter<'a, T, R> {
//     type Item = Sample<T>;

//     fn next(&mut self) -> Option<Self::Item> {
//         let n = self.residuals.len();
//         if n == 0 {
//             return Some(Sample::new(Vec::new()));
//         }

//         self.buffer.clear();
//         self.buffer.reserve_exact(n);

//         unsafe {
//             self.buffer.set_len(n);
//             let out_ptr = self.buffer.as_mut_ptr();
//             let res_ptr = self.residuals.as_ptr();
//             let fit_ptr = self.fitted.as_ptr();

//             // Vectorized reconstruction: y*_i = ŷ_i + w_i * e_i
//             // Unrolled loop for better ILP (Instruction-Level Parallelism)
//             let mut i = 0;

//             if let Some(ref weights) = self.precomputed_weights {
//                 // Precomputed weights path (faster when centering/scaling needed)
//                 let weights_ptr = weights.as_ptr();
//                 while i + 3 < n {
//                     let w0 = *weights_ptr.add(i);
//                     let w1 = *weights_ptr.add(i + 1);
//                     let w2 = *weights_ptr.add(i + 2);
//                     let w3 = *weights_ptr.add(i + 3);

//                     let e0 = *res_ptr.add(i);
//                     let e1 = *res_ptr.add(i + 1);
//                     let e2 = *res_ptr.add(i + 2);
//                     let e3 = *res_ptr.add(i + 3);

//                     let f0 = *fit_ptr.add(i);
//                     let f1 = *fit_ptr.add(i + 1);
//                     let f2 = *fit_ptr.add(i + 2);
//                     let f3 = *fit_ptr.add(i + 3);

//                     *out_ptr.add(i) = f0 + w0 * e0;
//                     *out_ptr.add(i + 1) = f1 + w1 * e1;
//                     *out_ptr.add(i + 2) = f2 + w2 * e2;
//                     *out_ptr.add(i + 3) = f3 + w3 * e3;
//                     i += 4;
//                 }

//                 while i < n {
//                     let w = *weights_ptr.add(i);
//                     let e = *res_ptr.add(i);
//                     let f = *fit_ptr.add(i);
//                     *out_ptr.add(i) = f + w * e;
//                     i += 1;
//                 }
//             } else {
//                 // On-the-fly weight generation path (faster when no centering/scaling)
//                 while i + 3 < n {
//                     let w0 = self.sample_weight();
//                     let w1 = self.sample_weight();
//                     let w2 = self.sample_weight();
//                     let w3 = self.sample_weight();

//                     let e0 = *res_ptr.add(i);
//                     let e1 = *res_ptr.add(i + 1);
//                     let e2 = *res_ptr.add(i + 2);
//                     let e3 = *res_ptr.add(i + 3);

//                     let f0 = *fit_ptr.add(i);
//                     let f1 = *fit_ptr.add(i + 1);
//                     let f2 = *fit_ptr.add(i + 2);
//                     let f3 = *fit_ptr.add(i + 3);

//                     *out_ptr.add(i) = f0 + w0 * e0;
//                     *out_ptr.add(i + 1) = f1 + w1 * e1;
//                     *out_ptr.add(i + 2) = f2 + w2 * e2;
//                     *out_ptr.add(i + 3) = f3 + w3 * e3;
//                     i += 4;
//                 }

//                 while i < n {
//                     let w = self.sample_weight();
//                     let e = *res_ptr.add(i);
//                     let f = *fit_ptr.add(i);
//                     *out_ptr.add(i) = f + w * e;
//                     i += 1;
//                 }
//             }
//         }

//         Some(Sample::new(std::mem::take(&mut self.buffer)))
//     }
// }

// // Convenience constructors for common econometric applications
// impl<R: Rng> WildBootstrap<R> {
//     /// Optimal wild bootstrap for linear regression with heteroskedasticity
//     /// Uses Mammen weights (optimal finite-sample properties)
//     pub fn linear_regression(rng: R) -> Self {
//         Self {
//             rng,
//             weight_dist: WeightDist::Mammen,
//             center_weights: true,
//             scale_weights: true,
//         }
//     }

//     /// Double/debiased machine learning bootstrap
//     /// Rademacher weights preferred for cross-fitting stability
//     pub fn double_ml(rng: R) -> Self {
//         Self {
//             rng,
//             weight_dist: WeightDist::Rademacher,
//             center_weights: true,
//             scale_weights: true,
//         }
//     }

//     /// Instrumental variables (2SLS) with heteroskedastic errors
//     /// Webb weights handle skewed reduced-form errors
//     pub fn instrumental_variables(rng: R) -> Self {
//         Self {
//             rng,
//             weight_dist: WeightDist::Webb,
//             center_weights: true,
//             scale_weights: true,
//         }
//     }

//     /// Nonparametric regression with kernel smoothing
//     /// Normal weights for asymptotic equivalence to smoothed bootstrap
//     pub fn nonparametric(rng: R) -> Self {
//         Self {
//             rng,
//             weight_dist: WeightDist::Normal,
//             center_weights: true,
//             scale_weights: true,
//         }
//     }
// }

// // Helper trait for numeric operations (simplified for illustration)
// trait Numeric: Copy + std::ops::Add<Output = Self> + std::ops::Mul<Output = Self> + std::ops::Sub<Output = Self> {
//     fn from(f: f64) -> Self;
//     fn zero() -> Self;
// }

// impl Numeric for f64 {
//     fn from(f: f64) -> Self { f }
//     fn zero() -> Self { 0.0 }
// }

// impl Numeric for f32 {
//     fn from(f: f64) -> Self { f as f32 }
//     fn zero() -> Self { 0.0 }
// }
