use num_traits::{Float, FromPrimitive, float::TotalOrder};
use crate::{CDF, Interval, QuantileInterval, Re, SE};

use super::Statistic;

#[derive(Debug, Clone)]
pub struct StudentizedBootstrap<Stat, InnerResampler, OuterResampler> {
    statistic: Stat,
    se: SE<Stat, InnerResampler>,
    resampler: OuterResampler,
    samples: usize,
    confidence: f64,
}

impl<Stat, InnerResampler, OuterResampler> StudentizedBootstrap<Stat, InnerResampler, OuterResampler> {
    pub fn new(
        statistic: Stat,
        se: SE<Stat, InnerResampler>,
        resampler: OuterResampler,
        samples: usize,
        confidence: f64,
    ) -> Self {
        debug_assert!((0.0..1.0).contains(&confidence));
        Self {
            statistic,
            se,
            resampler,
            samples,
            confidence,
        }
    }
}

impl<D, T, Stat, InnerResampler, OuterResampler> Statistic<D, Interval<T>>
    for StudentizedBootstrap<Stat, InnerResampler, OuterResampler>
where
    D: AsRef<[T]>,
    T: Float + FromPrimitive,
    Stat: Statistic<D, T>,
    InnerResampler: Re<D, Item = D>,
    OuterResampler: Re<D, Item = D>,
{
    fn compute(&self, data: &D) -> Interval<T> {
        let theta_hat = self.statistic.compute(data);
        let se_theta_hat = self.se.compute(data);
        if se_theta_hat.is_nan() || se_theta_hat.is_zero() {
            return Interval::nan();
        }

        let t_star: Vec<T> = self.resampler
            .re(data)
            .take(self.samples)
            .filter_map(|resample| {
                let theta_star = self.statistic.compute(&resample);
                let se_theta_star = self.se.compute(&resample);

                if se_theta_star.is_zero() || theta_star.is_nan() || se_theta_star.is_nan() {
                    None
                } else {
                    Some((theta_star - theta_hat) / se_theta_star)
                }
            })
            .collect();

        if t_star.len() < 2 {
            return Interval::nan();
        }

        // CRITICAL: ECDF must handle floats via total_cmp (see below)
        let ecdf = CDF.compute(&t_star);
        let (t_lower, t_upper) = QuantileInterval::percentile(self.confidence)
            .compute(&ecdf);

        let lower = theta_hat - t_upper * se_theta_hat;
        let upper = theta_hat - t_lower * se_theta_hat;

        Interval::new(lower, upper)
            .estimate(theta_hat)
            .confidence(self.confidence)
    }
}

// // Convenience constructors for common use cases
// impl<R> StudentizedBootstrap<Mean, JackknifeSE<Mean>, Bootstrap<R>>
// where
//     R: rand::Rng + Clone,
// {
//     /// 95% studentized bootstrap CI for the mean using jackknife SE.
//     pub fn mean_jackknife(resampler: Bootstrap<R>, samples: usize) -> Self {
//         Self::new(
//             Mean,
//             JackknifeSE::new(Mean),
//             resampler,
//             samples,
//             0.95,
//         )
//     }
// }
