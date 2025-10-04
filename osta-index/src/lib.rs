//! A library for working with indices, slices, and vectors with specific constraints.
//! Heavily inspired by `rustc_index`.

#![feature(extend_one)]
#![feature(step_trait)]

mod idx;
mod slice;
mod vec;

pub use idx::{Idx, IntoSliceIndex};
pub use slice::IndexSlice;
pub use vec::IndexVec;
