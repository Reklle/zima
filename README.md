[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)]()
[![Crates.io](https://img.shields.io/crates/v/zima.svg)](https://crates.io/crates/zima)

## What is Zima?

Zima is an attempt to create a modern crate for the needs of applied statistics. It includes:
- **Basic statistical estimators** (mean, variance, skewness, quantiles, etc.)
- **Resampling methods** (jackknife, bootstrap, subsampling, etc.)
- **Hypothesis tests** (D'Agostino, permutation test)

## WARNING

**Extremely unstable!** The API is far from final—it will definitely mutate many times.
**Lack of docs!** The code is currently totally sloppy. This will be fixed after the first non-alpha release.

## The design

### Declarative

Most statistical APIs treat estimators as functions: `var(sample)`. But real-world usage often requires configuration (e.g., `ddof` for variance), validation, or stateful setup.
This imposes an essential condition: *ideologically, estimators are not functions, but structures*. In Zima, every estimator is a struct you configure once and reuse.

### Abstract

Scalar statistics is a relic of computational convenience. Real-world datasets are inherently multidimensional: sensor arrays, financial instruments, image patches, and latent representations. The core principle is to avoid hardcoded computation types like `f32` or `f64`. This is zerocost thanks to static dispatch—but it leads to longer *initial* compilation time due to monomorphization. Subsequent rebuilds stay fast because Cargo caches monomorphized artifacts. See section Adaptive fast below.

```rust
impl<D, T> Statistic<D, T> for Mean
where
    D: AsRef<[T]>,
    T: Vector,
{
    fn compute(&self, data: &D) -> T { ... }
}
```

### Adaptive fast

Total time contains three components:

- **How fast the code will be (re)writen?** The code is modular and this allow writing a custom feature easily. The real problem in the following:
- **How fast the code will be compiled?**
- **How fast the code will be computed?**


There are two main scenarios in scientific computation: prototyping and production-scale analysis.In the first case the bottleneck is compilation/rebuild speed. In the second one it's raw computation speed.

Rust already handles this elegantly:

```toml
[profile.dev]
opt-level = 1
incremental = true

[profile.release]
opt-level = 3
incremental = false
```

First build takes time, but subsequent rebuilds are fast. Dependencies stay optimized even in dev builds.

### Distribution = Measurement = Resampling
Resampling procedures, probability distributions, and laboratory measurements all share the same `trait`.

## Code examples

### Bias Correction with Jackknife

```rust
let statistic = Variance { ddof: 1 };
let n = sample.len() as f32;

// Compute jackknife replicates
let estimated: Sample<f32> = Jackknife::new()
    .re(&sample)
    .map(|res| statistic.compute(&res))
    .collect();

// Original (biased) estimate
let biased = statistic.compute(&sample);

// Jackknife bias-corrected estimate
let unbiased = n * biased - (n - 1.0) * Mean.compute(&estimated);
```

### Hypothesis Testing

```rust
let sample: Sample<f32> = Sample::read("data/sample.csv")?;
println!("Loaded {} values", sample.len());

// Prints pretty table
println!("First of all, check normality of data;\n{}",
    DagostinoPearson::default().compute(&sample)
);
```
