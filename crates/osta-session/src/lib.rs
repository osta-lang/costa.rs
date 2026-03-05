#![allow(internal_features)]
#![feature(rustc_attrs)]
#![feature(step_trait)]
#![feature(allocator_api)]

use crate::interner::{BumpAllocator, Interner};

pub mod interner;

pub struct Session {
    pub interner: Interner<BumpAllocator>,
}

impl Session {
    pub fn new() -> Self {
        let interner = Interner::new();
        Self { interner }
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}
