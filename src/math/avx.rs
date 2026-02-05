// use std::simd::{f32x16, Simd};


// trait Projective: Module<T, const N: usize> {
//     fn project(self) -> [T; N]
// }

// /// derive Chunked wants trait Projective
// #[derive(Chunked)]
// struct

// we have struct X

// pub struct ChunkedF32<D>
// where
//     D: AsRef<[X]>,
// {
//     data: D,
//     index: usize,
// }

// impl<D> From<XChanked> for D
// where
//     D: AsRef<[X]>,
// {

// }


// /// Iterator over 16-element SIMD chunks of f32 data.
// ///
// /// Yields `f32x16` vectors for full chunks and provides access to remainder elements.
// /// Works with any type implementing `AsRef<[f32]>` (e.g., `Vec<f32>`, slices, arrays).
// #[derive(Debug, Clone)]
// pub struct ChunkedF32<D>
// where
//     D: AsRef<[f32]>,
// {
//     data: D,
//     index: usize,
// }

// impl<D> ChunkedF32<D>
// where
//     D: AsRef<[f32]>,
// {
//     /// Creates a new chunked iterator from any f32 slice source.
//     #[inline]
//     pub fn new(data: D) -> Self {
//         Self { data, index: 0 }
//     }

//     /// Returns the remaining elements that don't fill a complete SIMD vector.
//     #[inline]
//     pub fn remainder(&self) -> &[f32] {
//         &self.data.as_ref()[self.index..]
//     }

//     /// Returns the total number of elements in the underlying data.
//     #[inline]
//     pub fn len(&self) -> usize {
//         self.data.as_ref().len()
//     }

//     /// Returns true if there are no elements left (including remainder).
//     #[inline]
//     pub fn is_empty(&self) -> bool {
//         self.index >= self.len()
//     }
// }

// impl<D> Iterator for ChunkedF32<D>
// where
//     D: AsRef<[f32]>,
// {
//     type Item = f32x16;

//     #[inline]
//     fn next(&mut self) -> Option<Self::Item> {
//         let slice = self.data.as_ref();
//         if self.index + 16 > slice.len() {
//             return None;
//         }

//         // SAFETY: We just checked bounds for 16 elements
//         let chunk = f32x16::from_slice(&slice[self.index..self.index + 16]);
//         self.index += 16;
//         Some(*chunk)
//     }

//     #[inline]
//     fn size_hint(&self) -> (usize, Option<usize>) {
//         let remaining = self.data.as_ref().len() - self.index;
//         let chunks = remaining / 16;
//         (chunks, Some(chunks))
//     }
// }
