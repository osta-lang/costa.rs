#![allow(internal_features)]
#![feature(rustc_attrs)]
#![feature(step_trait)]

use osta_index::new_index;

new_index!(MyIdx: u32, max = 0xFFFF_FFFA);

#[test]
fn forward_iteration() {
    let range = MyIdx::from_ty(1)..MyIdx::from_ty(4);
    assert_eq!(
        range.collect::<Vec<_>>(),
        [MyIdx::from_ty(1), MyIdx::from_ty(2), MyIdx::from_ty(3)]
    )
}

#[test]
fn backward_iteration() {
    let range = MyIdx::from_ty(1)..MyIdx::from_ty(4);
    assert_eq!(
        range.rev().collect::<Vec<_>>(),
        [MyIdx::from_ty(3), MyIdx::from_ty(2), MyIdx::from_ty(1)]
    )
}

#[test]
fn range_count() {
    let range = MyIdx::from_ty(1)..MyIdx::from_ty(4);
    assert_eq!(range.count(), 3);
}

#[test]
fn range_size_hint() {
    let range = MyIdx::from_ty(1)..MyIdx::from_ty(4);
    assert_eq!(range.size_hint(), (3, Some(3)));
}
