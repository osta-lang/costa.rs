use std::fmt::Debug;
use std::hash::Hash;
use std::slice::SliceIndex;

pub trait Idx: Copy + Clone + Debug + PartialEq + Eq + Hash {
    fn new(idx: usize) -> Self;

    fn index(self) -> usize;

    #[inline]
    fn increment(&mut self, by: usize) {
        *self = self.plus(by);
    }

    #[inline]
    fn plus(self, by: usize) -> Self {
        Self::new(self.index() + by)
    }
}

impl Idx for usize {
    fn new(idx: usize) -> Self {
        idx
    }

    fn index(self) -> usize {
        self
    }
}

impl Idx for u32 {
    fn new(idx: usize) -> Self {
        assert!(idx < u32::MAX as usize);
        idx as u32
    }

    fn index(self) -> usize {
        self as usize
    }
}

pub trait IntoSliceIndex<I, T: ?Sized> {
    type Output: SliceIndex<T>;

    fn into_slice_idx(self) -> Self::Output;
}

impl<I: Idx, T> IntoSliceIndex<I, [T]> for I {
    type Output = usize;

    #[inline]
    fn into_slice_idx(self) -> Self::Output {
        self.index()
    }
}

impl<I: Idx, T> IntoSliceIndex<I, [T]> for std::ops::Range<I> {
    type Output = std::ops::Range<usize>;

    fn into_slice_idx(self) -> Self::Output {
        self.start.index()..self.end.index()
    }
}

impl<I: Idx, T> IntoSliceIndex<I, [T]> for std::ops::RangeFrom<I> {
    type Output = std::ops::RangeFrom<usize>;

    fn into_slice_idx(self) -> Self::Output {
        self.start.index()..
    }
}

impl<I: Idx, T> IntoSliceIndex<I, [T]> for std::ops::RangeTo<I> {
    type Output = std::ops::RangeTo<usize>;

    fn into_slice_idx(self) -> Self::Output {
        ..self.end.index()
    }
}

impl<I: Idx, T> IntoSliceIndex<I, [T]> for std::ops::RangeFull {
    type Output = std::ops::RangeFull;

    fn into_slice_idx(self) -> Self::Output {
        self
    }
}

impl<I: Idx, T> IntoSliceIndex<I, [T]> for std::ops::RangeInclusive<I> {
    type Output = std::ops::RangeInclusive<usize>;

    fn into_slice_idx(self) -> Self::Output {
        self.start().index()..=self.end().index()
    }
}

impl<I: Idx, T> IntoSliceIndex<I, [T]> for std::ops::RangeToInclusive<I> {
    type Output = std::ops::RangeToInclusive<usize>;

    fn into_slice_idx(self) -> Self::Output {
        ..=self.end.index()
    }
}

#[macro_export]
macro_rules! new_index {
    ($vis:vis $name:ident: $ty:ty, start = $start:literal, end = $end:literal, derive = [$($derive:ident),*]) => {
        #[repr(transparent)]
        #[rustc_layout_scalar_valid_range_start($start)]
        #[rustc_layout_scalar_valid_range_end($end)]
        #[derive($($derive),*)]
        $vis struct $name($ty);

        impl $name {
            pub const MIN: $ty = $start;
            pub const MAX: $ty = $end;
            pub const START: Self = Self::from_ty(Self::MIN);

            #[inline]
            pub const fn from_ty(index: $ty) -> Self {
                assert!(index >= $start && index <= $end);
                unsafe { Self(index) }
            }
        }

        impl $crate::Idx for $name {
            #[inline]
            fn new(idx: usize) -> Self {
                Self::from_ty(idx as $ty)
            }

            #[inline]
            fn index(self) -> usize {
                (self.0 - $start) as usize
            }
        }

        impl ::std::ops::Add<$ty> for $name {
            type Output = Self;

            #[inline]
            fn add(self, rhs: $ty) -> Self::Output {
                Self::from_ty(self.0 + rhs)
            }
        }

        impl ::std::ops::Sub<$ty> for $name {
            type Output = Self;

            #[inline]
            fn sub(self, rhs: $ty) -> Self::Output {
                Self::from_ty(self.0 - rhs)
            }
        }

        impl ::std::iter::Step for $name {
            fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
                <usize as ::std::iter::Step>::steps_between(
                    &$crate::Idx::index(*start),
                    &$crate::Idx::index(*end),
                )
            }

            fn forward_checked(start: Self, count: usize) -> Option<Self> {
                $crate::Idx::index(start).checked_add(count).map($crate::Idx::new)
            }

            fn backward_checked(start: Self, count: usize) -> Option<Self> {
                $crate::Idx::index(start).checked_sub(count).map($crate::Idx::new)
            }
        }
    };
    ($vis:vis $name:ident: $ty:ty, max = $max:literal, derive = [$($derive:ident),*]) => {
        new_index!($vis $name: $ty, start = 0, end = $max, derive = [$($derive),*]);
    };
    ($vis:vis $name:ident: $ty:ty, start = $start:literal, end = $end:literal $(, extra = [$($derive:ident),*])?) => {
        new_index!($vis $name: $ty, start = $start, end = $end, derive = [Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash $($(, $derive)*)?]);
    };
    ($vis:vis $name:ident: $ty:ty, max = $max:literal $(, extra = [$($derive:ident),*])?) => {
        new_index!($vis $name: $ty, start = 0, end = $max, extra = [$($($derive)*)?]);
    };
    ($vis:vis $name:ident: u32 $(, derive = [$($derive:ident),*])?) => {
        new_index!($vis $name: u32, start = 0, end = 0xFFFFFF00 $(, derive = [$($derive),*])?);
    };
    ($vis:vis $name:ident: u32 $(, extra = [$($derive:ident),*])?) => {
        new_index!($vis $name: u32, start = 0, end = 0xFFFFFF00 $(, extra = [$($derive),*])?);
    };
    ($vis:vis $name:ident: usize $(, derive = [$($derive:ident),*])?) => {
        new_index!($vis $name: usize, start = 0, end = 0xFFFFFFFFFFFFFF00 $(, derive = [$($derive),*])?);
    };
    ($vis:vis $name:ident: usize $(, extra = [$($derive:ident),*])?) => {
        new_index!($vis $name: usize, start = 0, end = 0xFFFFFFFFFFFFFF00 $(, extra = [$($derive),*])?);
    };
    ($vis:vis $name:ident: u32, min = $min:literal $(, derive = [$($derive:ident),*])?) => {
        new_index!($vis $name: u32, start = $min, end = 0xFFFFFFFF $(, derive = [$($derive),*])?);
    };
    ($vis:vis $name:ident: u32, min = $min:literal $(, extra = [$($derive:ident),*])?) => {
        new_index!($vis $name: u32, start = $min, end = 0xFFFFFFFF $(, extra = [$($derive),*])?);
    };
    ($vis:vis $name:ident: usize, min = $min:literal $(, derive = [$($derive:ident),*])?) => {
        new_index!($vis $name: usize, start = $min, end = 0xFFFFFFFFFFFFFFFF $(, derive = [$($derive),*])?);
    };
    ($vis:vis $name:ident: usize, min = $min:literal $(, extra = [$($derive:ident),*])?) => {
        new_index!($vis $name: usize, start = $min, end = 0xFFFFFFFFFFFFFFFF $(, extra = [$($derive),*])?);
    };
}
