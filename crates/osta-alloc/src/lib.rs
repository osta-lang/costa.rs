#![feature(allocator_api)]

use bumpalo::Bump;
use std::alloc::{AllocError, Allocator, Global, Layout};
use std::ptr::NonNull;

#[repr(transparent)]
pub struct BumpAllocator(Bump);

impl BumpAllocator {
    pub fn new() -> Self {
        BumpAllocator(Bump::new())
    }
}

unsafe impl Allocator for BumpAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let bump: &Bump = &self.0;
        Allocator::allocate(&bump, layout)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        let bump: &Bump = &self.0;
        unsafe {
            bump.deallocate(ptr, layout);
        };
    }
}

impl Default for BumpAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct ErasedAllocator<A: Allocator>(NonNull<A>);

impl<A: Allocator> ErasedAllocator<A> {
    pub fn new(allocator: &A) -> Self {
        Self(NonNull::from(allocator))
    }
}

unsafe impl<A: Allocator> Allocator for ErasedAllocator<A> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        unsafe { self.0.as_ref().allocate(layout) }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        unsafe {
            self.0.as_ref().deallocate(ptr, layout);
        }
    }
}

pub trait AllocatorExt: Allocator + Sized {
    fn allocate<T>(&self) -> Result<NonNull<T>, AllocError> {
        let layout = Layout::new::<T>();
        Allocator::allocate(self, layout)
            .map(|ptr| unsafe { NonNull::new_unchecked(ptr.as_ptr() as *mut T) })
    }

    fn vec<T>(&self) -> AllocVec<T, ErasedAllocator<Self>> {
        Vec::new_in_allocator(ErasedAllocator::new(self))
    }
}

impl<A: Allocator> AllocatorExt for A {}

pub trait VecExt<T>: Sized {
    fn new_in_allocator<'alloc, A: 'alloc + Allocator>(allocator: A) -> Vec<T, A> {
        Vec::new_in(allocator)
    }
}

impl<T> VecExt<T> for Vec<T> {}

pub type AllocVec<T, A = Global> = Vec<T, A>;

#[macro_export]
macro_rules! alloc_vec {
    ($allocator: expr $(, [])?) => {
        $crate::AllocatorExt::vec($allocator)
    };
    ($allocator: expr, [$x: expr; $n: expr]) => {{
        // Can't use this because SpecFromElem is private
        // <T as ::std::vec::SpecFromElem>::from_elem($x, $n, $allocator)
        let mut vec = $crate::alloc_vec!($allocator);
        for _ in 0..($n) {
            vec.push($x);
        }
        vec
    }};
    ($allocator: expr, [$($x:expr),+ $(,)?] $(,)?) => {{
        let mut vec = $crate::alloc_vec!($allocator);
        $(vec.push($x);)+
        vec
    }};
}

#[cfg(test)]
mod tests {
    use crate::{AllocatorExt, BumpAllocator};

    #[test]
    fn bump() {
        let arena = BumpAllocator::new();

        for _ in 0..u8::MAX {
            let mut vec = arena.vec();
            for i in 0..u8::MAX {
                vec.push(i);
            }
        }
    }
}
