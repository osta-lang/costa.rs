use osta_session::Session;

#[test]
fn interner() {
    let mut session = Session::create();

    let id1 = session.get_or_intern("hello");
    let id2 = session.get_or_intern("world");
    let id3 = session.get_or_intern("hello");

    assert_eq!(id1, id3);
    assert_eq!(session.resolve_intern(id1), "hello");
    assert_eq!(session.resolve_intern(id2), "world");
}
