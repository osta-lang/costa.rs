#[macro_use]
mod common;

use osta_lexer::TokenKind;

#[test]
fn grouping() {
    init_lexer!(src, lexer, "( ) { } [ ]");

    assert_next!(src, lexer, TOKEN, TokenKind::LParen, "(");
    assert_next!(src, lexer, TOKEN, TokenKind::RParen, ")");
    assert_next!(src, lexer, TOKEN, TokenKind::LBrace, "{");
    assert_next!(src, lexer, TOKEN, TokenKind::RBrace, "}");
    assert_next!(src, lexer, TOKEN, TokenKind::LBracket, "[");
    assert_next!(src, lexer, TOKEN, TokenKind::RBracket, "]");

    assert_next!(src, lexer, EOF);
}

#[test]
fn access() {
    init_lexer!(src, lexer, ". ::");

    assert_next!(src, lexer, TOKEN, TokenKind::Dot, ".");
    assert_next!(src, lexer, TOKEN, TokenKind::DoubleColon, "::");

    assert_next!(src, lexer, EOF);
}

#[test]
fn separation() {
    init_lexer!(src, lexer, ", ; : ->");

    assert_next!(src, lexer, TOKEN, TokenKind::Comma, ",");
    assert_next!(src, lexer, TOKEN, TokenKind::Semicolon, ";");
    assert_next!(src, lexer, TOKEN, TokenKind::Colon, ":");
    assert_next!(src, lexer, TOKEN, TokenKind::Arrow, "->");

    assert_next!(src, lexer, EOF);
}

#[test]
fn others() {
    init_lexer!(src, lexer, "... #");

    assert_next!(src, lexer, TOKEN, TokenKind::Ellipsis, "...");
    assert_next!(src, lexer, TOKEN, TokenKind::Pound, "#");

    assert_next!(src, lexer, EOF);
}
