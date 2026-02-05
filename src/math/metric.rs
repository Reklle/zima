// use num_traits::Float;
// use std::ops::{Add, Mul};

// trait Vector:
//     Add<Self, Output = Self> +
//     Sub<Self, Output = Self> +
//     Mul<Self::Field, Output = Self> +
//     Sized
// {
//     type Field: Float + Copy;
//     fn zero() -> Self;
// }

// trait Metric<V>
// where
//     V: Vector,
// {

//     fn sqrt(&self) -> impl Vielbein<V>;
//     fn dot(&self, v: V, w: V) -> V::Field;
//     fn det(&self) -> V::Field;
//     fn dvol(&self) -> V::Field
//     where
//         V::Field: Float,
//     {
//         self.det().sqrt()
//     }

//     fn norm(&self, v: V) -> V::Field
//     where
//         V: Copy,
//         V::Field: Float,
//     {
//         self.dot(v, v).sqrt()
//     }

//     fn angle(&self, v: V, w: V) -> V::Field
//     where
//         V: Copy,
//         V::Field: Float,
//     {
//         self.dot(v, w) / (self.norm(v) * self.norm(w))
//     }
// }

// trait Sym<V>
// where
//     V: Vector,
// {
//     fn metric(&self) -> impl Metric<V>;
//     fn otimes(v: V, w: V) -> Self;
// }



// trait Vielbein<V>
// where
//     V: Vector,
// {
//     ///  Induce metric
//     fn metric(&self) -> impl Metric<V>;
// }
