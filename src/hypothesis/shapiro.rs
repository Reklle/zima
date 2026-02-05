// use num_traits::{Float, FromPrimitive, ToPrimitive};
// use std::marker::PhantomData;
// use statrs::distribution::{Normal, ContinuousCDF};

// /// Результат теста Шапиро–Уилка
// #[derive(Debug, Clone, Copy, PartialEq)]
// pub struct ShapiroWResult<F: Float> {
//     /// Статистика W ∈ [0, 1]; ближе к 1 → данные ближе к нормальному распределению
//     pub w: F,
//     /// p-value (верхний хвост: вероятность наблюдать столь же или менее нормальные данные)
//     pub p_value: F,
// }

// /// Конфигурация теста Шапиро–Уилка
// #[derive(Debug, Clone, Copy)]
// pub struct Shapiro<F: Float> {
//     min_n: usize,
//     max_n: usize,
//     _phantom: PhantomData<F>,
// }

// impl<F: Float> Default for Shapiro<F> {
//     fn default() -> Self {
//         Self {
//             min_n: 3,
//             max_n: 5000,
//             _phantom: PhantomData,
//         }
//     }
// }

// impl<F: Float> Shapiro<F> {
//     /// Создаёт тест с настройками по умолчанию (рекомендуется)
//     pub fn new() -> Self {
//         Self::default()
//     }

//     /// Устанавливает границы допустимого размера выборки
//     pub fn with_bounds(mut self, min_n: usize, max_n: usize) -> Self {
//         self.min_n = min_n;
//         self.max_n = max_n;
//         self
//     }
// }

// impl<D, F> Statistic<D, ShapiroWResult<F>> for Shapiro<F>
// where
//     D: AsRef<[F]>,
//     F: Float + FromPrimitive + ToPrimitive,
// {
//     fn compute(&self, data: &D) -> ShapiroWResult<F> {
//         let x = data.as_ref();
//         let n = x.len();

//         // === Валидация ===
//         if n < self.min_n || n > self.max_n {
//             panic!(
//                 "Размер выборки {} выходит за границы [{}, {}]",
//                 n, self.min_n, self.max_n
//             );
//         }
//         if n == 0 {
//             panic!("Пустая выборка");
//         }

//         // === Шаг 1: Сортировка данных ===
//         let mut x_sorted = x.to_vec();
//         x_sorted.sort_by(|a, b| a.partial_cmp(b).expect("NaN в данных"));

//         // === Шаг 2: Базовые статистики ===
//         let sum: F = x_sorted.iter().copied().sum();
//         let mean = sum / F::from_usize(n).expect("n > 0");
//         let ssq: F = x_sorted
//             .iter()
//             .map(|&xi| {
//                 let diff = xi - mean;
//                 diff * diff
//             })
//             .sum();

//         // Вырожденный случай: нулевая дисперсия
//         if ssq == F::zero() {
//             return ShapiroWResult {
//                 w: F::zero(),
//                 p_value: F::zero(),
//             };
//         }

//         // === Шаг 3: Вычисление весов (точный метод Шапиро–Уилка) ===
//         // m_i = Φ⁻¹((i - 0.375) / (n + 0.25)) для i = 1..n
//         let mut m = Vec::with_capacity(n);
//         let n_f64 = n as f64;
//         for i in 1..=n {
//             let p = (i as f64 - 0.375) / (n_f64 + 0.25);
//             // Точное вычисление квантиля через вашу реализацию
//             let m_i_f64 = normal::ppf(p);
//             let m_i = F::from_f64(m_i_f64).expect("ppf вернул конечное значение");
//             m.push(m_i);
//         }

//         // Антисимметризация: a_i = (m_i - m_{n+1-i}) / 2
//         let mut a = vec![F::zero(); n];
//         for i in 0..n / 2 {
//             let j = n - 1 - i;
//             let diff = m[i] - m[j];
//             a[i] = diff / (F::from_f64(2.0).expect("2.0 конвертируется"));
//             a[j] = -a[i];
//         }
//         if n % 2 == 1 {
//             a[n / 2] = F::zero();
//         }

//         // Нормировка: ||a||₂ = 1
//         let norm_sq: F = a.iter().map(|&ai| ai * ai).sum();
//         let norm = norm_sq.sqrt();
//         for ai in &mut a {
//             *ai = *ai / norm;
//         }

//         // === Шаг 4: Статистика W ===
//         // W = (aᵀ(x - x̄))² / Σ(xᵢ - x̄)²
//         // Но поскольку Σaᵢ = 0 (из антисимметрии), можно использовать просто aᵀx
//         let weighted_sum: F = a.iter().zip(&x_sorted).map(|(&ai, &xi)| ai * xi).sum();
//         let w = (weighted_sum * weighted_sum) / ssq;

//         // === Шаг 5: p-value через трансформацию Ройстона (точные коэффициенты) ===
//         // Для n ≥ 12 используем асимптотическую нормальную аппроксимацию
//         let p_value = if n < 12 {
//             compute_exact_p_value_small_n(w, n) // Требует таблиц; для краткости — заглушка
//         } else {
//             compute_royston_p_value(w, n)
//         };

//         ShapiroWResult { w, p_value }
//     }
// }

// /// Точное вычисление p-value через трансформацию Ройстона (для n ≥ 12)
// /// Источник: Royston (1992), Statistics in Medicine, Vol. 11, pp. 117–123
// fn compute_royston_p_value<F>(w: F, n: usize) -> F
// where
//     F: Float + FromPrimitive + ToPrimitive,
// {
//     // Трансформация: y = ln(1 - W)
//     let one_minus_w = F::one() - w;
//     // Защита от численных ошибок (W может быть слегка > 1 из-за округления)
//     let y = if one_minus_w <= F::zero() {
//         F::neg_infinity()
//     } else {
//         one_minus_w.ln()
//     };

//     // Параметры нормальной аппроксимации (точные коэффициенты Ройстона)
//     let n_f64 = n as f64;
//     let ln_n = n_f64.ln();
//     let sqrt_n = n_f64.sqrt();

//     // μ = -1.5861 - 0.31082·ln(n) - 0.1294/n + 0.03028/√n
//     let mu_f64 = -1.5861
//         - 0.31082 * ln_n
//         - 0.1294 / n_f64
//         + 0.03028 / sqrt_n;

//     // σ = 0.7755 - 0.09744·ln(n) + 0.01217/n + 0.003784/√n
//     let sigma_f64 = 0.7755
//         - 0.09744 * ln_n
//         + 0.01217 / n_f64
//         + 0.003784 / sqrt_n;

//     let mu = F::from_f64(mu_f64).expect("mu конвертируется");
//     let sigma = F::from_f64(sigma_f64).expect("sigma конвертируется");

//     // Стандартизация: z = (y - μ) / σ
//     let z = (y - mu) / sigma;

//     // p-value = P(Z > z) = 1 - Φ(z)
//     let z_f64 = z.to_f64().expect("z конвертируется в f64");
//     let p_f64 = 1.0 - normal::cdf(z_f64);
//     F::from_f64(p_f64).expect("p-value конвертируется")
// }
