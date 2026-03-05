use crate::idx::Idx;
use crate::slice::IndexSlice;
use std::borrow::{Borrow, BorrowMut};
use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut, RangeBounds};
use std::vec::IntoIter;

#[repr(transparent)]
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct IndexVec<I: Idx, T> {
    pub(crate) raw: Vec<T>,
    _marker: std::marker::PhantomData<fn(&I)>,
}

impl<I: Idx, T> IndexVec<I, T> {
    #[inline]
    pub const fn new() -> Self {
        IndexVec::from_raw(Vec::new())
    }

    #[inline]
    pub const fn from_raw(raw: Vec<T>) -> Self {
        Self { raw, _marker: std::marker::PhantomData }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        IndexVec::from_raw(Vec::with_capacity(capacity))
    }

    #[inline]
    pub fn from_elem(elem: T, count: usize) -> Self
    where
        T: Clone,
    {
        IndexVec::from_raw(vec![elem; count])
    }

    #[inline]
    pub fn from_fn(func: impl FnMut(I) -> T, count: usize) -> Self {
        let _ = I::new(count); // OPTIMIZATION HINT
        IndexVec::from_raw((0..count).map(I::new).map(func).collect())
    }

    #[inline]
    pub fn as_slice(&self) -> &IndexSlice<I, T> {
        IndexSlice::from_raw(&self.raw)
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut IndexSlice<I, T> {
        IndexSlice::from_raw_mut(&mut self.raw)
    }

    #[inline]
    pub fn push(&mut self, value: T) -> I {
        let idx = self.next_index();
        self.raw.push(value);
        idx
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        self.raw.pop()
    }

    #[inline]
    pub fn into_enumerate(self) -> impl DoubleEndedIterator<Item = (I, T)> + ExactSizeIterator {
        let _ = I::new(self.raw.len()); // OPTIMIZATION HINT
        self.raw
            .into_iter()
            .enumerate()
            .map(|(i, v)| (I::new(i), v))
    }

    #[inline]
    pub fn drain<R: RangeBounds<usize>>(&mut self, range: R) -> impl Iterator<Item = T> {
        self.raw.drain(range)
    }

    #[inline]
    pub fn drain_enumerate<R: RangeBounds<usize>>(
        &mut self,
        range: R,
    ) -> impl Iterator<Item = (I, T)> {
        let begin = match range.start_bound() {
            std::ops::Bound::Included(&start) => start,
            std::ops::Bound::Excluded(&start) => start + 1,
            std::ops::Bound::Unbounded => 0,
        };
        let _ = I::new(self.raw.len()); // OPTIMIZATION HINT
        self.raw
            .drain(range)
            .enumerate()
            .map(move |(i, v)| (I::new(begin + i), v))
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.raw.shrink_to_fit()
    }

    #[inline]
    pub fn truncate(&mut self, len: usize) {
        self.raw.truncate(len)
    }

    #[inline]
    pub fn get_or_grow_with(&mut self, idx: I, func: impl FnMut() -> T) -> &mut T {
        if self.len() <= idx.index() {
            self.resize_to(idx, func);
        }
        &mut self[idx]
    }

    #[inline]
    pub fn resize(&mut self, new_len: usize, value: T)
    where
        T: Clone,
    {
        self.raw.resize(new_len, value);
    }

    #[inline]
    pub fn resize_to(&mut self, new_len: I, func: impl FnMut() -> T) {
        self.raw.resize_with(new_len.index() + 1, func);
    }

    #[inline]
    pub fn append(&mut self, other: &mut Self) {
        self.raw.append(&mut other.raw);
    }
}

impl<I: Idx, T> IndexVec<I, Option<T>> {
    #[inline]
    pub fn insert(&mut self, index: I, value: T) -> Option<T> {
        self.get_or_grow_with(index, || None).replace(value)
    }

    #[inline]
    pub fn get_or_insert_with(&mut self, index: I, func: impl FnMut() -> T) -> &mut T {
        self.get_or_grow_with(index, || None)
            .get_or_insert_with(func)
    }

    #[inline]
    pub fn remove(&mut self, index: I) -> Option<T> {
        self.get_mut(index)?.take()
    }

    #[inline]
    pub fn contains(&self, index: I) -> bool {
        self.get(index).is_some()
    }
}

impl<I: Idx, T: Debug> Debug for IndexVec<I, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.raw.fmt(f)
    }
}

impl<I: Idx, T> Deref for IndexVec<I, T> {
    type Target = IndexSlice<I, T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<I: Idx, T> DerefMut for IndexVec<I, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<I: Idx, T> Borrow<IndexSlice<I, T>> for IndexVec<I, T> {
    #[inline]
    fn borrow(&self) -> &IndexSlice<I, T> {
        self
    }
}

impl<I: Idx, T> BorrowMut<IndexSlice<I, T>> for IndexVec<I, T> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut IndexSlice<I, T> {
        self
    }
}

impl<I: Idx, T> Extend<T> for IndexVec<I, T> {
    #[inline]
    fn extend<It: IntoIterator<Item = T>>(&mut self, iter: It) {
        self.raw.extend(iter);
    }

    #[inline]
    fn extend_one(&mut self, item: T) {
        self.raw.push(item);
    }

    #[inline]
    fn extend_reserve(&mut self, additional: usize) {
        self.raw.reserve(additional);
    }
}

impl<I: Idx, T> FromIterator<T> for IndexVec<I, T> {
    #[inline]
    fn from_iter<It: IntoIterator<Item = T>>(iter: It) -> Self {
        IndexVec::from_raw(Vec::from_iter(iter))
    }
}

impl<I: Idx, T> IntoIterator for IndexVec<I, T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.raw.into_iter()
    }
}

impl<'a, I: Idx, T> IntoIterator for &'a IndexVec<I, T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.raw.iter()
    }
}

impl<'a, I: Idx, T> IntoIterator for &'a mut IndexVec<I, T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.raw.iter_mut()
    }
}

impl<I: Idx, T> Default for IndexVec<I, T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<I: Idx, T, const N: usize> From<[T; N]> for IndexVec<I, T> {
    #[inline]
    fn from(array: [T; N]) -> Self {
        IndexVec::from_raw(array.into())
    }
}

unsafe impl<I: Idx, T> Send for IndexVec<I, T> {}
