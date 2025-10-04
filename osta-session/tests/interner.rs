use osta_session::interner::Interner;

#[test]
fn interner() {
    let mut interner = Interner::new();

    let id1 = interner.get_or_intern("hello");
    let id2 = interner.get_or_intern("world");
    let id3 = interner.get_or_intern("hello");

    assert_eq!(id1, id3);
    assert_eq!(interner.resolve(id1), "hello");
    assert_eq!(interner.resolve(id2), "world");
}
