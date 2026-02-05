//! Basic linear algebra traits
//!
//! # Motivation
//! Projective modules over a ring `R` represent a natural commonality for modern computing:
//! vector spaces are too narrow a class for modern applications.
//! For example, block matrices are often used—they are the same matrices,
//! but in a module over a ring of matrices. This simple idea cannot be formalized in the language of vector spaces.
//! But for computation, the class of all modules is too broad:
//! a basis doesn't always exist. This is why projective modules are important—they admit coordinate systems.
//! All fundamental computational procedures of linear algebra rely not on linear independence of a basis,
//! but on linearity of coordinates, and thus generalize naturally to projective modules.
//!
//! For applied statistics we require scaling vectors by `n`, `n.sqrt()`, etc. Algebraically,
//! this means the scalar ring contains ℝ as a subring.
//! While `num-traits::Float` is not the minimal requirement, it provides a practical
//! computational model for this subring.

use num_traits::{Float, FromPrimitive};
use std::ops::{Add, Div, Mul, Sub};

/// Vector space over a float.
///
/// The `Copy` bound restricts this trait to stack-allocated types.
pub trait Vector:
    Add<Self, Output = Self>
    + Sub<Self, Output = Self>
    + Mul<Self::Field, Output = Self>
    + Div<Self::Field, Output = Self>
    + Sized
    + Copy
{
    type Field: Float + FromPrimitive + Copy;

    /// Zero vector of this space.
    fn zero() -> Self;

    /// Convert `usize` to the scalar field.
    /// Ppanics on overflow.
    #[inline(always)]
    fn from_usize(u: usize) -> Self::Field {
        <Self::Field as FromPrimitive>::from_usize(u)
            .expect("usize-to-float conversion failed")
    }
}

/// Coordinate representation of a projective module.
///
/// `N` is the rank of the corresponding free module summand.
/// `BaseType` is the scalar type used for coordinates (typically `Float`).
pub trait Projective<const N: usize = 1> {
    type BaseType;

    /// Project the element onto its coordinate representation.
    fn to_array(self) -> [Self::BaseType; N];

    /// Reconstruct the element from coordinates.
    fn from_array(array: [Self::BaseType; N]) -> Self;
}

// --- Implementations for primitives ----------------------------------------------

impl<F> Vector for F
where
    F: Float + FromPrimitive + Copy,
{
    type Field = F;

    #[inline(always)]
    fn zero() -> Self {
        F::zero()
    }
}

impl<F> Projective<1> for F
where
    F: Float + FromPrimitive + Copy,
{
    type BaseType = F;

    #[inline(always)]
    fn to_array(self) -> [F; 1] {
        [self]
    }

    #[inline(always)]
    fn from_array(array: [F; 1]) -> Self {
        array[0]
    }
}

// --- Left/Right module structure --------------------------------------

// The distinction between left and right modules is essential for noncommutative rings
// (quaternions ℍ, matrix rings Mₙ(ℝ), operator algebras). In such contexts,
// r⋅v and v⋅r are fundamentally different operations.
//
// WARNING: Implementations MUST satisfy module axioms:
//   1 ⋅ v = v                       (unit)
//   r ⋅ (u + v) = r⋅u + r⋅v          (left distributivity)
//  (r + s) ⋅ v = r⋅v + s⋅v           (right distributivity)
//  (r ⋅ s) ⋅ v = r ⋅ (s ⋅ v)          (associativity of action)
//
// Violation does not cause algorithmic UB


// /// WARNING: the action (multiplication) must respect the algebraic structure of R.
// pub trait RightModule<R>:
//     Vector +
//     Mul<R, Output = Self>
// { }
//
// /// WARNING: the left action (multiplication) must respect the algebraic structure of L.
// pub trait LeftModule<L>:
//     Vector
// where
//     L: Mul<Self, Output = Self>
// { }
