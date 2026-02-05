use num_traits::{Float, FromPrimitive};
use statrs::distribution::{ChiSquared, ContinuousCDF};
use crate::statistics::*;
use crate::statistics::Statistic;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DagostinoPearsonResult<F: Float> {
    // K² ~ χ²(2)
    pub statistic: F,
    pub p_value: F,
    // Выборочная асимметрия (не смещённая оценка)
    pub skewness: F,
    // Избыточный эксцесс (0 при нормальности)
    pub kurtosis: F,
}

#[derive(Debug, Clone, Copy)]
pub struct DagostinoPearson {
    min_n: usize,
}

impl Default for DagostinoPearson {
    fn default() -> Self {
        Self { min_n: 8 }
    }
}

impl DagostinoPearson {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_min_n(mut self, min_n: usize) -> Self {
        self.min_n = min_n;
        self
    }
}

impl<D, F> Statistic<D, DagostinoPearsonResult<F>> for DagostinoPearson
where
    D: AsRef<[F]>,
    F: Float + FromPrimitive + Copy,
{
    fn compute(&self, data: &D) -> DagostinoPearsonResult<F> {
        let slice = data.as_ref();
        let n = slice.len();

        // === Валидация входных данных ===
        debug_assert!(n >= self.min_n, "Sample size {} < minimum {}", n, self.min_n);
        debug_assert!(!slice.iter().any(|x| x.is_nan()), "Data contains NaN values");

        // Проверка нулевой дисперсии (защита от деления на ноль в асимметрии/эксцессе)
        let mean = Mean.compute(data);
        let variance = Variance::default().compute(data);
        debug_assert!(
            variance > F::epsilon(),
            "Zero variance detected (all values identical)"
        );

        // === Шаг 1: Вычисление асимметрии и эксцесса ===
        // ВАЖНО: Используем выборочные (несмещённые) оценки
        let skewness = Skewness::unbiased().compute(data);
        let kurtosis = Kurtosis::unbiased().compute(data);

        // === Шаг 2: Нормализация асимметрии (трансформация Д'Агостино 1970/1973) ===
        let z_skew = normalize_skewness(skewness, n);

        // === Шаг 3: Нормализация эксцесса (трансформация Анскомба-Глинн 1983) ===
        let z_kurt = normalize_kurtosis(kurtosis, n);

        // === Шаг 4: Объединённая статистика ===
        // K² = Z₁² + Z₂² ~ χ²(2) при справедливости H₀
        let statistic = z_skew * z_skew + z_kurt * z_kurt;

        // === Шаг 5: p-value через точную χ²(2) ===
        let p_value = chi2_sf::<F>(statistic, 2);

        DagostinoPearsonResult {
            statistic,
            p_value,
            skewness,
            kurtosis,
        }
    }
}

fn normalize_skewness<F>(g1: F, n: usize) -> F
where
    F: Float + FromPrimitive,
{
    // Трансформация Д'Агостино (1970, исправленная 1973) - точная формула
    let n_f = F::from_usize(n).expect("n must be representable");

    // Шаг 1: y = g₁ · √[(n+1)(n+3) / (6(n-2))]
    let n_plus_1 = n_f + F::one();
    let n_plus_3 = n_f + F::from_usize(3).expect("3");
    let n_minus_2 = n_f - F::from_usize(2).expect("2");
    let six = F::from_usize(6).expect("6");

    let y = g1 * ((n_plus_1 * n_plus_3) / (six * n_minus_2)).sqrt();

    // Шаг 2: β₂ = 3(n²+27n-70)(n+1)(n+3) / [(n-2)(n+5)(n+7)(n+9)]
    let n_sq = n_f * n_f;
    let b2_num = F::from_usize(3).expect("3")
        * (n_sq + F::from_usize(27).expect("27") * n_f - F::from_usize(70).expect("70"))
        * n_plus_1
        * n_plus_3;

    let n_plus_5 = n_f + F::from_usize(5).expect("5");
    let n_plus_7 = n_f + F::from_usize(7).expect("7");
    let n_plus_9 = n_f + F::from_usize(9).expect("9");

    let b2_den = n_minus_2 * n_plus_5 * n_plus_7 * n_plus_9;
    let b2 = b2_num / b2_den;

    // Шаг 3: w² = √[2(β₂ - 1)] - 1  (формула 1973 коррекции)
    let two = F::from_usize(2).expect("2");
    // Численная защита: β₂ может быть < 1 из-за округления при малых n
    let b2_adj = if b2 > F::one() { b2 } else { F::one() + F::epsilon() };
    let w_sq = (two * (b2_adj - F::one())).sqrt() - F::one();

    // Шаг 4: δ = 1 / √[ln(w)], где w = √(w²)
    // Численная защита: w_sq должно быть > 0
    let w_sq_adj = if w_sq > F::zero() { w_sq } else { F::epsilon() };
    let w = w_sq_adj.sqrt();
    let delta = F::one() / w.ln().sqrt();

    // Шаг 5: Z₁ = δ · ln(y + √(y² + 1))
    let z1 = delta * (y + (y * y + F::one()).sqrt()).ln();

    z1
}

fn normalize_kurtosis<F>(kurtosis: F, n: usize) -> F
where
    F: Float + FromPrimitive,
{
    // Трансформация Анскомба-Глинн (1983) - точная формула
    let n_f = F::from_usize(n).expect("n must be representable");
    let three = F::from_usize(3).expect("3");
    let b2 = kurtosis;

    // Шаг 2: Ожидаемый эксцесс при нормальности: E = 3(n-1)/(n+1)
    let n_minus_1 = n_f - F::one();
    let n_plus_1 = n_f + F::one();
    let e = three * n_minus_1 / n_plus_1;

    // Шаг 3: Дисперсия при нормальности
    let n_minus_2 = n_f - F::from_usize(2).expect("2");
    let n_minus_3 = n_f - F::from_usize(3).expect("3");
    let n_plus_3 = n_f + F::from_usize(3).expect("3");
    let n_plus_5 = n_f + F::from_usize(5).expect("5");
    let twenty_four = F::from_usize(24).expect("24");

    let var_num = twenty_four * n_f * n_minus_2 * n_minus_3;
    let var_den = n_plus_1 * n_plus_1 * n_plus_3 * n_plus_5;
    let var = var_num / var_den;

    // Шаг 4: Стандартизированный статистик
    let x = (b2 - e) / var.sqrt();

    // Шаг 5: Параметр трансформации a
    let six = F::from_usize(6).expect("6");
    let eight = F::from_usize(8).expect("8");
    let sqrt_6 = six.sqrt();
    let ratio = (n_plus_1 / n_minus_1).sqrt();
    let a = six + (eight / sqrt_6) * (ratio - F::one());

    // Шаг 6: Финальная трансформация
    let nine = F::from_usize(9).expect("9");
    let two = F::from_usize(2).expect("2");

    let term_a = two / (nine * a);
    // Численная защита для корня кубического
    let numerator = F::one() - two / a;
    let denom_inner = a / (a - two);
    // Гарантируем положительность под корнем
    let denom_inner_adj = if denom_inner > F::zero() { denom_inner } else { F::epsilon() };
    let denominator = F::one() + x * denom_inner_adj.sqrt();

    let ratio_cbrt = if denominator != F::zero() {
        (numerator / denominator).cbrt()
    } else {
        F::zero()
    };

    let z2 = (F::one() - term_a - ratio_cbrt) / term_a.sqrt();

    z2
}

// Точная функция выживания хи-квадрат через statrs
fn chi2_sf<F>(x: F, df: u64) -> F
where
    F: Float + FromPrimitive,
{
    let x_f64 = x.to_f64().expect("x must be representable as f64");
    let chi2 = ChiSquared::new(df as f64).expect("df must be positive");
    let sf = chi2.sf(x_f64);
    F::from_f64(sf).expect("sf must be representable")
}
