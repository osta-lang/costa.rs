#![allow(internal_features)]
#![feature(rustc_attrs)]
#![feature(step_trait)]

use osta_index::new_index;

#[test]
fn lower_niche() {
    new_index!(pub MyIdx: u32, min = 5);

    assert_eq!(size_of::<MyIdx>(), size_of::<u32>());

    assert_eq!(size_of::<Option<MyIdx>>(), size_of::<u32>());
    assert_eq!(size_of::<Option<Option<MyIdx>>>(), size_of::<u32>());
    assert_eq!(
        size_of::<Option<Option<Option<MyIdx>>>>(),
        size_of::<u32>()
    );
    assert_eq!(
        size_of::<Option<Option<Option<Option<MyIdx>>>>>(),
        size_of::<u32>()
    );
    assert_eq!(
        size_of::<Option<Option<Option<Option<Option<MyIdx>>>>>>(),
        size_of::<u32>()
    );

    assert_eq!(
        size_of::<Option<Option<Option<Option<Option<Option<MyIdx>>>>>>>(),
        size_of::<u64>()
    );

    let opt: Option<MyIdx> = None;
    let opt: u32 = unsafe { std::mem::transmute(opt) };
    assert_eq!(opt, 0);
}

#[test]
fn upper_niche() {
    new_index!(pub MyIdx: u32, max = 0xFFFF_FFFA);

    assert_eq!(size_of::<MyIdx>(), size_of::<u32>());

    assert_eq!(size_of::<Option<MyIdx>>(), size_of::<u32>());
    assert_eq!(size_of::<Option<Option<MyIdx>>>(), size_of::<u32>());
    assert_eq!(
        size_of::<Option<Option<Option<MyIdx>>>>(),
        size_of::<u32>()
    );
    assert_eq!(
        size_of::<Option<Option<Option<Option<MyIdx>>>>>(),
        size_of::<u32>()
    );
    assert_eq!(
        size_of::<Option<Option<Option<Option<Option<MyIdx>>>>>>(),
        size_of::<u32>()
    );

    assert_eq!(
        size_of::<Option<Option<Option<Option<Option<Option<MyIdx>>>>>>>(),
        size_of::<u64>()
    );

    let opt: Option<MyIdx> = None;
    let opt: u32 = unsafe { std::mem::transmute(opt) };
    assert_eq!(opt, 0xFFFF_FFFB);
}
