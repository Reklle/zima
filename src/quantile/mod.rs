// use num_traits::{Float, NumOps};

// use crate::EmpiricalCDF;

// /// There is a lot of different methods to estimate quantile,
// pub trait Quantile {
//     fn quantile<F>(&self, cdf: EmpiricalCDF<F>) -> impl Fn(F) -> F
//     where
//         F: Float + Copy;
// }

// struct HyndmanFan {

// }

// struct HarrellDavis {

// }

// struct SfakianakisVerginis {

// }


// trait Interpolate<X, Y> {
//     fn interpolation<D>(&self, data: D) -> impl Fn(X) -> Y
//     where
//         D: AsRef<[(X, Y)]>;
// }

// struct Interpolator<X, Y> {
//     data: Vec<(X, Y)>
// }

// impl Interpolator {
//     fn interpolate<I>(interp: I) -> impl Fn(X) -> Y
//     where
//         I: Interpolate
//     {
//         interp.interpolation(self.data)
//     }
// }
