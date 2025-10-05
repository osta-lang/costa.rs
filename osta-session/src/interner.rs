use bumpalo::Bump;
use osta_index::{new_index, IndexVec};
use std::alloc::{handle_alloc_error, AllocError, Allocator, Layout};
use std::collections::HashMap;
use std::mem::transmute;
use std::ptr::{copy_nonoverlapping, NonNull};
use std::slice::from_raw_parts_mut;
use std::str::from_utf8_unchecked_mut;

new_index!(pub InternId: u32);

pub struct Interner<A: Allocator> {
    allocator: A,
    indices: IndexVec<InternId, NonNull<str>>,
    map: HashMap<&'static str, InternId>,
}

impl Interner<BumpAllocator> {
    pub fn new() -> Self {
        Self::with_bump(Bump::new())
    }

    pub fn with_bump(bump: Bump) -> Self {
        Self::with_allocator(BumpAllocator(bump))
    }
}

impl<A: Allocator> Interner<A> {
    pub fn with_allocator(allocator: A) -> Self {
        Self { allocator, indices: IndexVec::new(), map: HashMap::new() }
    }

    pub fn resolve(&self, id: InternId) -> &str {
        unsafe { self.indices[id].as_ref() }
    }

    pub fn get_or_intern(&mut self, s: &str) -> InternId {
        if let Some(&id) = self.map.get(s) {
            return id;
        }
        self.intern(s)
    }

    fn intern(&mut self, s: &str) -> InternId {
        let bytes = s.as_bytes();
        let layout = Layout::from_size_align(bytes.len(), align_of::<u8>())
            .expect("Failed to create layout");
        let ptr = self.allocator.allocate(layout);
        let ptr = match ptr {
            Ok(memory) => memory.as_ptr() as *mut u8,
            Err(_) => handle_alloc_error(layout),
        };
        unsafe {
            copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
            let slice = from_raw_parts_mut(ptr, bytes.len());
            let str_ptr = from_utf8_unchecked_mut(slice);
            let nn_str = NonNull::new_unchecked(str_ptr);

            let id = self.indices.push(nn_str);
            let static_str: &'static str = transmute(str_ptr);
            let pre = self.map.insert(static_str, id);

            debug_assert!(pre.is_none(), "String was already interned");

            id
        }
    }
}

impl Default for Interner<BumpAllocator> {
    fn default() -> Self {
        Self::new()
    }
}

#[repr(transparent)]
pub struct BumpAllocator(Bump);

unsafe impl Allocator for BumpAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let bump: &Bump = &self.0;
        bump.allocate(layout)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        let bump: &Bump = &self.0;
        unsafe {
            bump.deallocate(ptr, layout);
        };
    }
}
