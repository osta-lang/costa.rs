use osta_lexer::{LexerErrorKind, TokenKind};

#[macro_use]
mod common;

#[test]
fn recovery() {
    init_lexer!(src, lexer, r#"\hello"#);

    match lexer.next() {
        Some(Err(err)) if matches!(err.kind, LexerErrorKind::UnknownToken) => {}
        _ => panic!("UnknownToken error was expected"),
    }

    assert_next!(src, lexer, TOKEN, TokenKind::Identifier, "hello")
}
