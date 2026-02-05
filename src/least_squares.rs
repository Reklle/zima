// use csv::Trim;
// use num_traits::{Float, real::Real};

// pub trait Statistic<D, T> {
//     fn compute(&self, data: &D) -> T;
// }

// trait JeffreysPrior<D, F> {
//     fn log_jeffreys(&self, data: &D) -> F;
// }

// trait LogPosterior<D, F> {
//     fn logp(&self, data: &D) -> F;
// }

// impl<T, D, F> LogPosterior<D, F> for T
// where
//     T: Statistic<D, F> + JeffreysPrior<D, F>,
//     F: Float,
// {
//     fn logp(&self, data: &D) -> F {
//         self.compute(data) + self.log_jeffreys(data)
//     }
// }


// struct Estimator<Model>
// where
//     Model: Statistic<P, D>
// {
//     model: Model,
//     data: D,
// }

// impl Estimator {
//     self.data
//         .iter()
//         .map(|v| f.compute())
// }

// struct ExpFit {
//     a: f32,
//     b: f32,
// }

// impl Statistic<f32, f32> for ExpFit {
//     fn compute(&self, data: f32) -> f32 {
//         (-data*self.b).exp()*a
//     }
// }

// impl JeffreysPrior for ExpFit {
//     fn logp(&self) -> f32 {
//         -self.a.log2()
//     }
// }
