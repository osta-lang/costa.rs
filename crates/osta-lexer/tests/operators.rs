#[macro_use]
mod common;

use osta_lexer::{Lexer, TokenKind};

#[test]
fn arithmetic() {
    init_lexer!(src, lexer, "+ - * / %");

    assert_next!(src, lexer, TOKEN, TokenKind::Plus, "+");
    assert_next!(src, lexer, TOKEN, TokenKind::Minus, "-");
    assert_next!(src, lexer, TOKEN, TokenKind::Star, "*");
    assert_next!(src, lexer, TOKEN, TokenKind::Slash, "/");
    assert_next!(src, lexer, TOKEN, TokenKind::Percent, "%");

    assert_next!(src, lexer, EOF);
}

#[test]
fn bitwise() {
    init_lexer!(src, lexer, "& | ^ << >> >>> ~");

    assert_next!(src, lexer, TOKEN, TokenKind::Ampersand, "&");
    assert_next!(src, lexer, TOKEN, TokenKind::Pipe, "|");
    assert_next!(src, lexer, TOKEN, TokenKind::Caret, "^");
    assert_next!(src, lexer, TOKEN, TokenKind::LShift, "<<");
    assert_next!(src, lexer, TOKEN, TokenKind::RShift, ">>");
    assert_next!(src, lexer, TOKEN, TokenKind::ARShift, ">>>");
    assert_next!(src, lexer, TOKEN, TokenKind::Tilde, "~");

    assert_next!(src, lexer, EOF);
}

#[test]
fn assignment() {
    init_lexer!(src, lexer, "= += -= *= /= %= &= |= ^= <<= >>= >>>=");

    assert_next!(src, lexer, TOKEN, TokenKind::Equal, "=");
    assert_next!(src, lexer, TOKEN, TokenKind::PlusEqual, "+=");
    assert_next!(src, lexer, TOKEN, TokenKind::MinusEqual, "-=");
    assert_next!(src, lexer, TOKEN, TokenKind::StarEqual, "*=");
    assert_next!(src, lexer, TOKEN, TokenKind::SlashEqual, "/=");
    assert_next!(src, lexer, TOKEN, TokenKind::PercentEqual, "%=");
    assert_next!(src, lexer, TOKEN, TokenKind::AmpersandEqual, "&=");
    assert_next!(src, lexer, TOKEN, TokenKind::PipeEqual, "|=");
    assert_next!(src, lexer, TOKEN, TokenKind::CaretEqual, "^=");
    assert_next!(src, lexer, TOKEN, TokenKind::LShiftEqual, "<<=");
    assert_next!(src, lexer, TOKEN, TokenKind::RShiftEqual, ">>=");
    assert_next!(src, lexer, TOKEN, TokenKind::ARShiftEqual, ">>>=");

    assert_next!(src, lexer, EOF);
}

#[test]
fn comparison() {
    init_lexer!(src, lexer, "== != < <= > >=");

    assert_next!(src, lexer, TOKEN, TokenKind::DoubleEqual, "==");
    assert_next!(src, lexer, TOKEN, TokenKind::NotEqual, "!=");
    assert_next!(src, lexer, TOKEN, TokenKind::Less, "<");
    assert_next!(src, lexer, TOKEN, TokenKind::LessEqual, "<=");
    assert_next!(src, lexer, TOKEN, TokenKind::Greater, ">");
    assert_next!(src, lexer, TOKEN, TokenKind::GreaterEqual, ">=");

    assert_next!(src, lexer, EOF);
}

#[test]
fn logical() {
    init_lexer!(src, lexer, "&& ||");

    assert_next!(src, lexer, TOKEN, TokenKind::DoubleAmpersand, "&&");
    assert_next!(src, lexer, TOKEN, TokenKind::DoublePipe, "||");

    assert_next!(src, lexer, EOF);
}

#[test]
fn range() {
    init_lexer!(src, lexer, ".. ..=");

    assert_next!(src, lexer, TOKEN, TokenKind::DoubleDot, "..");
    assert_next!(src, lexer, TOKEN, TokenKind::DoubleDotEqual, "..=");

    assert_next!(src, lexer, EOF);
}

#[test]
fn unary() {
    init_lexer!(src, lexer, "++ -- ! ?");

    assert_next!(src, lexer, TOKEN, TokenKind::DoublePlus, "++");
    assert_next!(src, lexer, TOKEN, TokenKind::DoubleMinus, "--");
    assert_next!(src, lexer, TOKEN, TokenKind::Bang, "!");
    assert_next!(src, lexer, TOKEN, TokenKind::Question, "?");

    assert_next!(src, lexer, EOF);
}
