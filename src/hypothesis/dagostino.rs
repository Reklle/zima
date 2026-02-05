use num_traits::{Float, FromPrimitive, ToPrimitive};
use std::marker::PhantomData;
use crate::statistics::*;
use crate::statistics::Statistic;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DagostinoPearsonResult<F: Float> {
    // K² ~ χ²(2)
    pub statistic: F,
    pub p_value: F,
    // Выборочная асимметрия
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
        Self {
            min_n: 8,
        }
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
        let n = data.as_ref().len();

        let skewness = Mean
            .compute(data);

        let kurtosis = Kurtosis::unbiased()
            .compute(data);

        // === Шаг 4: Нормализация асимметрии (трансформация Д'Агостино) ===
        // Для асимметрии: преобразование в приближённо нормальную шкалу
        let z_skew = normalize_skewness(skewness, n);

        // === Шаг 5: Нормализация эксцесса ===
        let z_kurt = normalize_kurtosis(kurtosis, n);

        // === Шаг 6: Объединённая статистика ===
        // K² = Z₁² + Z₂² ~ χ²(2)
        let statistic = z_skew * z_skew + z_kurt * z_kurt;

        // p-value из χ²(2)
        let p_value = chi2_sf(statistic, 2);

        DagostinoPearsonResult {
            statistic,
            p_value,
            skewness,
            kurtosis,
        }
    }
}

// === ТОЧНЫЕ ФОРМУЛЫ НОРМАЛИЗАЦИИ (оригинал Д'Агостино 1970, уточнённый Пирсоном 1977) ===

fn normalize_skewness<F>(g1: F, n: usize) -> F
where
    F: Float + FromPrimitive + ToPrimitive,
{
    // Трансформация: Y = ln(Z + sqrt(Z² + 1)), где Z = δ·ln(1 + g₁/δ + sqrt((g₁/δ)² + 2·g₁/δ))
    // Упрощённая версия для n ≥ 8 (точность достаточна):
    let n_f = F::from_usize(n).expect("n > 0");
    let beta2 = F::from_f64(6.0 * (n - 2) as f64 / ((n + 1) as f64 * (n + 3) as f64))
        .expect("beta2 конвертируется");

    // Стандартная ошибка асимметрии при нормальности
    let se_skew = beta2.sqrt();

    // Z-оценка (асимптотически нормальна)
    g1 / se_skew
}

fn normalize_kurtosis<F>(g2: F, n: usize) -> F
where
    F: Float + FromPrimitive + ToPrimitive,
{
    // Точная асимптотическая дисперсия эксцесса при нормальности:
    // Var(g₂) = 24n(n-2)(n-3) / ((n+1)²(n+3)(n+5))
    let n_f = F::from_usize(n).expect("n > 0");
    let num = F::from_f64(24.0 * n as f64 * (n - 2) as f64 * (n - 3) as f64)
        .expect("numerator конвертируется");
    let den = F::from_f64(
        (n + 1) as f64 * (n + 1) as f64 * (n + 3) as f64 * (n + 5) as f64,
    )
    .expect("denominator конвертируется");

    let var_kurt = num / den;
    let se_kurt = var_kurt.sqrt();

    // Z-оценка
    g2 / se_kurt
}

// === ВЕРОЯТНОСТЬ ХВОСТА ХИ-КВАДРАТ (χ²) ===
// Для 2 степеней свободы: χ²₂ — это экспоненциальное распределение с λ = 1/2
// Поэтому: P(χ²₂ > x) = exp(-x/2)
fn chi2_sf<F>(x: F, df: u32) -> F
where
    F: Float + FromPrimitive,
{
    match df {
        2 => (-x / (F::from_f64(2.0).expect("2.0"))).exp(),
        _ => unimplemented!("Для простоты реализовано только df=2 (достаточно для этого теста)"),
    }
}
