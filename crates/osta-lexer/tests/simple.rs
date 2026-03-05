#[macro_use]
mod common;

use osta_lexer::{Lexer, TokenKind};

#[test]
fn empty() {
    init_lexer!(src, lexer, "");

    assert_next!(src, lexer, EOF);
}

#[test]
fn simple() {
    init_lexer!(src, lexer, "let x = 42;");

    assert_next!(src, lexer, TOKEN, TokenKind::Let, "let");
    assert_next!(src, lexer, TOKEN, TokenKind::Identifier, "x");
    assert_next!(src, lexer, TOKEN, TokenKind::Equal, "=");
    assert_next!(src, lexer, TOKEN, TokenKind::DecInt, "42");
    assert_next!(src, lexer, TOKEN, TokenKind::Semicolon, ";");

    assert_next!(src, lexer, EOF);
}

#[test]
fn with_comments() {
    init_lexer!(
        src,
        lexer,
        "let x = 42; // this is a comment\n/* multi-line\ncomment */\nlet y = x + 1;"
    );

    assert_next!(src, lexer, TOKEN, TokenKind::Let, "let");
    assert_next!(src, lexer, TOKEN, TokenKind::Identifier, "x");
    assert_next!(src, lexer, TOKEN, TokenKind::Equal, "=");
    assert_next!(src, lexer, TOKEN, TokenKind::DecInt, "42");
    assert_next!(src, lexer, TOKEN, TokenKind::Semicolon, ";");
    assert_next!(src, lexer, TOKEN, TokenKind::Let, "let");
    assert_next!(src, lexer, TOKEN, TokenKind::Identifier, "y");
    assert_next!(src, lexer, TOKEN, TokenKind::Equal, "=");
    assert_next!(src, lexer, TOKEN, TokenKind::Identifier, "x");
    assert_next!(src, lexer, TOKEN, TokenKind::Plus, "+");
    assert_next!(src, lexer, TOKEN, TokenKind::DecInt, "1");
    assert_next!(src, lexer, TOKEN, TokenKind::Semicolon, ";");

    assert_next!(src, lexer, EOF);
}
