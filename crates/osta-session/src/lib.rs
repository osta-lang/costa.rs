#![allow(internal_features)]
#![feature(rustc_attrs)]
#![feature(step_trait)]
#![feature(allocator_api)]

use crate::interner::{InternId, Interner};
use osta_alloc::BumpAllocator;

pub mod interner;

pub type LongTermAllocator = BumpAllocator;
pub type ShortTermAllocator = BumpAllocator;

pub struct Session {
    /// Long-Term Allocator
    pub lta: LongTermAllocator,

    /// Short-Term Allocator
    pub sta: ShortTermAllocator,

    pub interner: Interner<LongTermAllocator>,
}

impl Session {
    pub fn new() -> Self {
        let lta = BumpAllocator::new();
        let sta = BumpAllocator::new();
        let interner = Interner::new_in(BumpAllocator::new());
        Self { lta, sta, interner }
    }

    pub fn get_or_intern(&mut self, s: &str) -> InternId {
        self.interner.get_or_intern(s)
    }

    pub fn resolve_intern(&self, id: InternId) -> &str {
        self.interner.resolve(id)
    }
}
