use rand::prelude::thread_rng;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rand::prelude::*; // Provides thread_rng()
use std::hint::black_box; // Modern black_box (Criterion 0.5+ deprecated its own)
use rand_xoshiro::Xoshiro256PlusPlus;
use zima::*;
use rand::SeedableRng;
const SAMPLE_SIZE: usize = 1_000;      // Fixed size for tests 2-4
const RESAMPLES: usize = 10_000;       // Bootstrap iterations

fn xrng() -> impl rand::Rng {
    <Xoshiro256PlusPlus as SeedableRng>::seed_from_u64(thread_rng().next_u64())
}

/// 1. VARIANCE COMPUTE (scaling test with multiple sizes)
fn bench_variance_compute(c: &mut Criterion) {
    let mut group = c.benchmark_group("variance/compute");
    group.throughput(Throughput::Elements(1));

    for &size in &[100, 1_000, 10_000] {
        let data: Vec<f32> = (0..size).map(|i| (i % 100) as f32).collect();
        let statistic = Variance::default(); // ddof=1

        group.bench_with_input(
            BenchmarkId::new("default", size),
            &data,
            |b, data| b.iter(|| black_box(statistic.compute(black_box(data)))),
        );
    }
    group.finish();
}

/// 2. JACKKNIFE STANDARD ERROR (Mean statistic)
fn bench_se_jackknife(c: &mut Criterion) {
    let sample: Vec<f32> = (0..SAMPLE_SIZE).map(|i| (i % 100) as f32).collect();

    c.bench_function("se/jackknife_mean", |b| {
        b.iter(|| black_box(SE::jackknife(black_box(&sample))))
    });
}

// /// 3. FULL BOOTSTRAP WITH STANDARD RNG (rand::thread_rng)
// fn bench_bootstrap_thread_rng(c: &mut Criterion) {
//     let sample: Vec<f32> = (0..SAMPLE_SIZE).map(|i| (i % 100) as f32).collect();
//     let statistic = Variance { ddof: 1 };

//     c.bench_function("bootstrap/thread_rng", |b| {
//         b.iter(|| {
//             let mut rng = rng_thread(); // Standard RNG
//             black_box(
//                 (0..RESAMPLES)
//                     .map(|_| {
//                         let resample: Vec<f32> = (0..sample.len())
//                             .map(|_| sample[rng.gen_range(0..sample.len())])
//                             .collect();
//                         statistic.compute(&resample)
//                     })
//                     .collect::<Vec<f64>>(),
//             )
//         })
//     });
// }

// /// 4. FULL BOOTSTRAP WITH FAST RNG (Xoshiro256++)
// fn bench_bootstrap_xoshiro(c: &mut Criterion) {
//     let sample: Vec<f32> = (0..SAMPLE_SIZE).map(|i| (i % 100) as f32).collect();
//     let statistic = Variance { ddof: 1 };

//     c.bench_function("bootstrap/xoshiro", |b| {
//         b.iter(|| {
//             let mut rng = rng_xoshiro(); // Fast RNG
//             black_box(
//                 (0..RESAMPLES)
//                     .map(|_| {
//                         let resample: Vec<f32> = (0..sample.len())
//                             .map(|_| sample[rng.gen_range(0..sample.len())])
//                             .collect();
//                         statistic.compute(&resample)
//                     })
//                     .collect::<Vec<f64>>(),
//             )
//         })
//     });
// }

// fn rn<R: Rng + SeedableRng>(r: R) {
//     r.
// }

/// 5. DIRECT COMPARISON: Your exact usage pattern
fn bench_your_pattern(c: &mut Criterion) {
    let sample: Sample<f32> = (0..SAMPLE_SIZE).map(|i| (i % 100) as f32).collect();
    let statistic = Variance { ddof: 1 };

    // Pattern A: thread_rng
    c.bench_function("pattern/thread_rng", |b| {
        b.iter(|| {
            let estimated: Sample<f32> = sample::Bootstrap::new(xrng())
                .re(&sample)
                .take(RESAMPLES)
                .map(|res| statistic.compute(&res))
                .collect();
            black_box(estimated)
        })
    });

    // Pattern B: xoshiro
    c.bench_function("pattern/xoshiro", |b| {
        b.iter(|| {
            let estimated: Sample<f32> = sample::Bootstrap::new(xrng())
                .re(&sample)
                .take(RESAMPLES)
                .map(|res| statistic.compute(&res))
                .collect();
            black_box(estimated)
        })
    });
}

criterion_group!(
    benches,
    bench_variance_compute,
    bench_se_jackknife,
    // bench_bootstrap_thread_rng,
    // bench_bootstrap_xoshiro,
    bench_your_pattern
);
criterion_main!(benches);
