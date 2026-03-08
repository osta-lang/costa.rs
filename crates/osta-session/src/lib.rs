#![allow(internal_features)]
#![feature(rustc_attrs)]
#![feature(step_trait)]
#![feature(allocator_api)]

use crate::interner::{InternId, Interner};
use osta_alloc::BumpAllocator;

pub mod interner;

pub type LongTermAllocator = BumpAllocator;
pub type ShortTermAllocator = BumpAllocator;

#[ouroboros::self_referencing]
pub struct Session {
    /// Long-Term Allocator
    pub lta: LongTermAllocator,

    /// Short-Term Allocator
    pub sta: ShortTermAllocator,

    #[borrows(sta)]
    #[not_covariant]
    pub interner: Interner<'this, BumpAllocator>,
}

impl Session {
    pub fn create() -> Self {
        let lta = BumpAllocator::new();
        let sta = BumpAllocator::new();
        Self::new(lta, sta, |sta| Interner::new_in(sta))
    }

    pub fn get_or_intern(&mut self, s: &str) -> InternId {
        self.with_interner_mut(|interner| interner.get_or_intern(s))
    }

    pub fn resolve_intern(&self, id: InternId) -> &str {
        self.with_interner(|interner| interner.resolve(id))
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::create()
    }
}
