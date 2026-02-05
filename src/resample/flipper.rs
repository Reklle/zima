use rand::Rng;
use crate::Sample;
use super::Re;

/// Customizable flipping strategy trait
pub trait Flip<T> {
    /// Flip `value` based on `control` flag (true = flip, false = identity)
    #[inline(always)]
    fn flip(&self, value: T, control: bool) -> T;
}

/// Default sign-flipping strategy for IEEE 754 floats using bit manipulation
#[derive(Clone, Copy, Default)]
pub struct SignBitFlip;

impl Flip<f32> for SignBitFlip {
    #[inline(always)]
    fn flip(&self, value: f32, control: bool) -> f32 {
        // Branchless mask via wrapping_neg (safe, zero-cost):
        // false → 0u32.wrapping_neg() = 0x0000_0000
        // true  → 1u32.wrapping_neg() = 0xFFFF_FFFF
        let mask = (control as u32).wrapping_neg() & 0x8000_0000;
        f32::from_bits(value.to_bits() ^ mask)
    }
}

impl Flip<f64> for SignBitFlip {
    #[inline(always)]
    fn flip(&self, value: f64, control: bool) -> f64 {
        let mask = (control as u64).wrapping_neg() & 0x8000_0000_0000_0000;
        f64::from_bits(value.to_bits() ^ mask)
    }
}

/// Multiplicative flip strategy (fallback for non-IEEE types)
#[derive(Clone, Copy, Default)]
pub struct MultiplyFlip;

impl<T> Flip<T> for MultiplyFlip
where
    T: Copy + std::ops::Neg<Output = T>,
{
    #[inline(always)]
    fn flip(&self, value: T, control: bool) -> T {
        if control { -value } else { value }
    }
}

#[derive(Clone, Copy)]
pub struct Flipper<R, F> {
    pub rng: R,
    pub flip: F,
}

impl<R: Rng> Flipper<R, SignBitFlip> {
    /// Create sign-flipping resampler for IEEE 754 floats (optimal bit manipulation)
    pub fn sign(rng: R) -> Self {
        Self {
            rng,
            flip: SignBitFlip,
        }
    }
}

impl<R: Rng, F> Flipper<R, F> {
    pub fn with_strategy(rng: R, flip: F) -> Self {
        Self { rng, flip }
    }
}

impl<T, R, F> Re<Sample<T>> for Flipper<R, F>
where
    R: Rng + Clone,
    F: Flip<T> + Clone,
    T: Copy,
{
    type Item = Sample<T>;

    fn re(&self, sample: &Sample<T>) -> impl Iterator<Item = Self::Item> {
        FlipperIter::new(&sample.data, self.rng.clone(), self.flip.clone())
    }
}

pub struct FlipperIter<'a, T, R: Rng, F: Flip<T>> {
    data: &'a [T],
    rng: R,
    flip: F,
    buffer: Vec<T>,
    /// Bit reservoir: holds 64 pre-generated random bits
    bit_reservoir: u64,
    /// Number of consumed bits (0..64)
    bits_consumed: u32,
}

impl<'a, T: Copy, R: Rng, F: Flip<T>> FlipperIter<'a, T, R, F> {
    fn new(data: &'a [T], rng: R, flip: F) -> Self {
        Self {
            buffer: Vec::with_capacity(data.len()),
            data,
            rng,
            flip,
            bit_reservoir: 0,
            bits_consumed: 64, // Force refill on first use
        }
    }
}

impl<'a, T: Copy, R: Rng, F: Flip<T>> Iterator for FlipperIter<'a, T, R, F> {
    type Item = Sample<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let n = self.data.len();
        if n == 0 {
            return Some(Sample::new(Vec::new()));
        }

        // Reuse buffer allocation
        if self.buffer.capacity() < n {
            self.buffer.reserve_exact(n);
        }

        unsafe {
            self.buffer.set_len(n);
            std::ptr::copy_nonoverlapping(self.data.as_ptr(), self.buffer.as_mut_ptr(), n);

            let ptr = self.buffer.as_mut_ptr();
            let mut i = 0;

            // 8-way unrolled loop consuming 8 bits per iteration
            // Borrow-checker safe: all state mutations explicit in loop
            while i + 7 < n {
                // Refill reservoir if needed (branch predicted perfectly)
                if self.bits_consumed > 56 {
                    self.bit_reservoir = self.rng.next_u64();
                    self.bits_consumed = 0;
                }

                // Extract 8 bits in parallel (branchless)
                let bits = self.bit_reservoir >> self.bits_consumed;
                self.bits_consumed += 8;

                // Unrolled flips using bit tests (no branches)
                *ptr.add(i) = self.flip.flip(*ptr.add(i), (bits & (1u64 << 0)) != 0);
                *ptr.add(i + 1) = self.flip.flip(*ptr.add(i + 1), (bits & (1u64 << 1)) != 0);
                *ptr.add(i + 2) = self.flip.flip(*ptr.add(i + 2), (bits & (1u64 << 2)) != 0);
                *ptr.add(i + 3) = self.flip.flip(*ptr.add(i + 3), (bits & (1u64 << 3)) != 0);
                *ptr.add(i + 4) = self.flip.flip(*ptr.add(i + 4), (bits & (1u64 << 4)) != 0);
                *ptr.add(i + 5) = self.flip.flip(*ptr.add(i + 5), (bits & (1u64 << 5)) != 0);
                *ptr.add(i + 6) = self.flip.flip(*ptr.add(i + 6), (bits & (1u64 << 6)) != 0);
                *ptr.add(i + 7) = self.flip.flip(*ptr.add(i + 7), (bits & (1u64 << 7)) != 0);
                i += 8;
            }

            // Handle remaining elements (0-7) - borrow-checker safe version
            // Explicit state mutation avoids simultaneous borrows
            while i < n {
                if self.bits_consumed >= 64 {
                    self.bit_reservoir = self.rng.next_u64();
                    self.bits_consumed = 0;
                }
                let flip_bit = (self.bit_reservoir >> self.bits_consumed) & 1;
                self.bits_consumed += 1;

                *ptr.add(i) = self.flip.flip(*ptr.add(i), flip_bit != 0);
                i += 1;
            }
        }

        Some(Sample::new(std::mem::take(&mut self.buffer)))
    }
}
