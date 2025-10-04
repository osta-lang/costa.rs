use crate::idx::{Idx, IntoSliceIndex};
use crate::vec::IndexVec;
use std::fmt::Debug;
use std::ops::{Index, IndexMut, RangeBounds};
use std::slice::{GetDisjointMutError, SliceIndex};

#[repr(transparent)]
#[derive(PartialEq, Eq, Hash)]
pub struct IndexSlice<I: Idx, T> {
    _marker: std::marker::PhantomData<fn(&I)>,
    pub(crate) raw: [T],
}

impl<I: Idx, T> IndexSlice<I, T> {
    #[inline]
    pub const fn empty<'a>() -> &'a Self {
        Self::from_raw(&[])
    }

    #[inline]
    pub const fn from_raw(raw: &[T]) -> &Self {
        let ptr: *const [T] = raw;
        // SAFETY: `IndexSlice` is `repr(transparent)` over `[T]`
        unsafe { &*(ptr as *const Self) }
    }

    #[inline]
    pub fn from_raw_mut(raw: &mut [T]) -> &mut Self {
        let ptr: *mut [T] = raw;
        // SAFETY: `IndexSlice` is `repr(transparent)` over `[T]`
        unsafe { &mut *(ptr as *mut Self) }
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.raw.len()
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    #[inline]
    pub fn next_index(&self) -> I {
        I::new(self.len())
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.raw.iter()
    }

    #[inline]
    pub fn enumerate(&self) -> impl DoubleEndedIterator<Item=(I, &T)> + ExactSizeIterator {
        let _ = I::new(self.len()); // OPTIMIZATION HINT
        self.raw.iter().enumerate().map(|(i, v)| (I::new(i), v))
    }

    #[inline]
    pub fn indicies(
        &self,
    ) -> impl DoubleEndedIterator<Item=I> + ExactSizeIterator + Clone + 'static {
        let _ = I::new(self.len()); // OPTIMIZATION HINT
        (0..self.len()).map(|i| I::new(i))
    }

    #[inline]
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.raw.iter_mut()
    }

    #[inline]
    pub fn enumerate_mut(
        &mut self,
    ) -> impl DoubleEndedIterator<Item=(I, &mut T)> + ExactSizeIterator {
        let _ = I::new(self.len()); // OPTIMIZATION HINT
        self.raw.iter_mut().enumerate().map(|(i, v)| (I::new(i), v))
    }

    #[inline]
    pub fn last_index(&self) -> Option<I> {
        self.len().checked_sub(1).map(|i| I::new(i))
    }

    #[inline]
    pub fn swap(&mut self, a: I, b: I) {
        self.raw.swap(a.index(), b.index())
    }

    #[inline]
    pub fn copy_within(
        &mut self,
        src: impl IntoSliceIndex<I, T, Output: RangeBounds<usize>>,
        dest: I,
    ) where
        T: Copy,
    {
        self.raw.copy_within(src.into_slice_idx(), dest.index());
    }

    #[inline]
    pub fn get<R: IntoSliceIndex<I, [T]>>(
        &self,
        index: R,
    ) -> Option<&<R::Output as SliceIndex<[T]>>::Output> {
        self.raw.get(index.into_slice_idx())
    }

    #[inline]
    pub fn get_mut<R: IntoSliceIndex<I, [T]>>(
        &mut self,
        index: R,
    ) -> Option<&mut <R::Output as SliceIndex<[T]>>::Output> {
        self.raw.get_mut(index.into_slice_idx())
    }

    #[inline]
    pub fn get_disjoint_mut<const N: usize>(
        &mut self,
        indices: [I; N],
    ) -> Result<[&mut T; N], GetDisjointMutError> {
        self.raw.get_disjoint_mut(indices.map(|i| i.index()))
    }

    #[inline]
    pub fn binary_search(&self, value: &T) -> Result<I, I>
    where
        T: Ord,
    {
        match self.raw.binary_search(value) {
            Ok(idx) => Ok(I::new(idx)),
            Err(idx) => Err(I::new(idx)),
        }
    }
}

impl<I: Idx, J: Idx> IndexSlice<I, J> {
    pub fn invert(&self) -> IndexVec<J, I> {
        debug_assert_eq!(
            self.iter().map(|x| x.index() as u128).sum::<u128>(),
            (0..self.len() as u128).sum::<u128>(),
            "The values aren't 0..N in input {self:?}",
        );

        let mut inverse = IndexVec::from_elem(Idx::new(0), self.len());
        for (i1, &i2) in self.enumerate() {
            inverse[i2] = i1;
        }

        debug_assert_eq!(
            inverse.iter().map(|x| x.index() as u128).sum::<u128>(),
            (0..inverse.len() as u128).sum::<u128>(),
            "The values aren't 0..N in result {self:?}",
        );

        inverse
    }
}

impl<I: Idx, T: Debug> Debug for IndexSlice<I, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.raw.fmt(f)
    }
}

impl<I: Idx, T, R: IntoSliceIndex<I, [T]>> Index<R> for IndexSlice<I, T> {
    type Output = <R::Output as SliceIndex<[T]>>::Output;

    #[inline]
    fn index(&self, index: R) -> &Self::Output {
        &self.raw[index.into_slice_idx()]
    }
}

impl<I: Idx, T, R: IntoSliceIndex<I, [T]>> IndexMut<R> for IndexSlice<I, T> {
    #[inline]
    fn index_mut(&mut self, index: R) -> &mut Self::Output {
        &mut self.raw[index.into_slice_idx()]
    }
}

impl<'a, I: Idx, T> IntoIterator for &'a IndexSlice<I, T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.raw.iter()
    }
}

impl<'a, I: Idx, T> IntoIterator for &'a mut IndexSlice<I, T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.raw.iter_mut()
    }
}

impl<I: Idx, T: Clone> ToOwned for IndexSlice<I, T> {
    type Owned = IndexVec<I, T>;

    fn to_owned(&self) -> Self::Owned {
        IndexVec::from_raw(self.raw.to_owned())
    }

    fn clone_into(&self, target: &mut Self::Owned) {
        self.raw.clone_into(&mut target.raw)
    }
}

impl<I: Idx, T> Default for &IndexSlice<I, T> {
    #[inline]
    fn default() -> Self {
        IndexSlice::from_raw(Default::default())
    }
}

impl<I: Idx, T> Default for &mut IndexSlice<I, T> {
    #[inline]
    fn default() -> Self {
        IndexSlice::from_raw_mut(Default::default())
    }
}

unsafe impl<I: Idx, T> Send for IndexSlice<I, T> where T: Send {}
