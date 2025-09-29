#[macro_use]
mod common;

use osta_lexer::{Lexer, TokenKind};

#[test]
fn identifier() {
    init_lexer!(src, lexer, "foo bar_baz _qux quux123");

    assert_next!(src, lexer, TOKEN, TokenKind::Identifier, "foo");
    assert_next!(src, lexer, TOKEN, TokenKind::Identifier, "bar_baz");
    assert_next!(src, lexer, TOKEN, TokenKind::Identifier, "_qux");
    assert_next!(src, lexer, TOKEN, TokenKind::Identifier, "quux123");

    assert_next!(src, lexer, EOF);
}

#[test]
fn macro_identifier() {
    init_lexer!(src, lexer, "@foo @bar_baz @qux @quux123");

    assert_next!(src, lexer, TOKEN, TokenKind::MacroIdentifier, "@foo");
    assert_next!(src, lexer, TOKEN, TokenKind::MacroIdentifier, "@bar_baz");
    assert_next!(src, lexer, TOKEN, TokenKind::MacroIdentifier, "@qux");
    assert_next!(src, lexer, TOKEN, TokenKind::MacroIdentifier, "@quux123");

    assert_next!(src, lexer, EOF);
}

#[test]
fn comptime_identifier() {
    init_lexer!(src, lexer, "#foo #bar_baz #qux #quux123");

    assert_next!(src, lexer, TOKEN, TokenKind::ComptimeIdentifier, "#foo");
    assert_next!(src, lexer, TOKEN, TokenKind::ComptimeIdentifier, "#bar_baz");
    assert_next!(src, lexer, TOKEN, TokenKind::ComptimeIdentifier, "#qux");
    assert_next!(src, lexer, TOKEN, TokenKind::ComptimeIdentifier, "#quux123");

    assert_next!(src, lexer, EOF);
}

#[test]
fn directive_identifier() {
    init_lexer!(src, lexer, "$foo $bar_baz $qux $quux123");

    assert_next!(src, lexer, TOKEN, TokenKind::DirectiveIdentifier, "$foo");
    assert_next!(src, lexer, TOKEN, TokenKind::DirectiveIdentifier, "$bar_baz");
    assert_next!(src, lexer, TOKEN, TokenKind::DirectiveIdentifier, "$qux");
    assert_next!(src, lexer, TOKEN, TokenKind::DirectiveIdentifier, "$quux123");

    assert_next!(src, lexer, EOF);
}

#[test]
fn lifetime_identifier() {
    init_lexer!(src, lexer, "'foo 'bar_baz 'qux 'quux123");

    assert_next!(src, lexer, TOKEN, TokenKind::LifetimeIdentifier, "'foo");
    assert_next!(src, lexer, TOKEN, TokenKind::LifetimeIdentifier, "'bar_baz");
    assert_next!(src, lexer, TOKEN, TokenKind::LifetimeIdentifier, "'qux");
    assert_next!(src, lexer, TOKEN, TokenKind::LifetimeIdentifier, "'quux123");

    assert_next!(src, lexer, EOF);
}

#[test]
fn integer() {
    init_lexer!(src, lexer, "0 42 01__234_567_890");

    assert_next!(src, lexer, TOKEN, TokenKind::DecInt, "0");
    assert_next!(src, lexer, TOKEN, TokenKind::DecInt, "42");
    assert_next!(src, lexer, TOKEN, TokenKind::DecInt, "01__234_567_890");

    assert_next!(src, lexer, EOF);
}

#[test]
fn binary_integer() {
    init_lexer!(src, lexer, "0b0 0B1 0b1010 0B1111_0000");

    assert_next!(src, lexer, TOKEN, TokenKind::BinInt, "0b0");
    assert_next!(src, lexer, TOKEN, TokenKind::BinInt, "0B1");
    assert_next!(src, lexer, TOKEN, TokenKind::BinInt, "0b1010");
    assert_next!(src, lexer, TOKEN, TokenKind::BinInt, "0B1111_0000");

    assert_next!(src, lexer, EOF);
}

#[test]
fn octal_integer() {
    init_lexer!(src, lexer, "0o0 0O7 0o1234 0O7654_3210");

    assert_next!(src, lexer, TOKEN, TokenKind::OctInt, "0o0");
    assert_next!(src, lexer, TOKEN, TokenKind::OctInt, "0O7");
    assert_next!(src, lexer, TOKEN, TokenKind::OctInt, "0o1234");
    assert_next!(src, lexer, TOKEN, TokenKind::OctInt, "0O7654_3210");

    assert_next!(src, lexer, EOF);
}

#[test]
fn hexadecimal_integer() {
    init_lexer!(src, lexer, "0x0 0Xf 0xDeadBeef 0XCAFE_BABE");

    assert_next!(src, lexer, TOKEN, TokenKind::HexInt, "0x0");
    assert_next!(src, lexer, TOKEN, TokenKind::HexInt, "0Xf");
    assert_next!(src, lexer, TOKEN, TokenKind::HexInt, "0xDeadBeef");
    assert_next!(src, lexer, TOKEN, TokenKind::HexInt, "0XCAFE_BABE");

    assert_next!(src, lexer, EOF);
}

#[test]
fn float() {
    init_lexer!(src, lexer, "0.0 3.14 27. 0.123456789");

    assert_next!(src, lexer, TOKEN, TokenKind::Float, "0.0");
    assert_next!(src, lexer, TOKEN, TokenKind::Float, "3.14");
    assert_next!(src, lexer, TOKEN, TokenKind::IntFloat, "27.");
    assert_next!(src, lexer, TOKEN, TokenKind::Float, "0.123456789");

    assert_next!(src, lexer, EOF);
}

#[test]
fn float_exponent() {
    init_lexer!(src, lexer, "1.0e10 3.14E-2 27e+5 6.022e23");

    assert_next!(src, lexer, TOKEN, TokenKind::FloatExp, "1.0e10");
    assert_next!(src, lexer, TOKEN, TokenKind::FloatExp, "3.14E-2");
    assert_next!(src, lexer, TOKEN, TokenKind::IntExp, "27e+5");
    assert_next!(src, lexer, TOKEN, TokenKind::FloatExp, "6.022e23");

    assert_next!(src, lexer, EOF);
}

#[test]
fn string() {
    init_lexer!(src, lexer, r#""hello" "world" "foo\"bar\"""#);

    assert_next!(src, lexer, TOKEN, TokenKind::String, r#""hello""#);
    assert_next!(src, lexer, TOKEN, TokenKind::String, r#""world""#);
    assert_next!(src, lexer, TOKEN, TokenKind::String, r#""foo\"bar\"""#);

    assert_next!(src, lexer, EOF);
}

#[test]
fn raw_string() {
    init_lexer!(src, lexer, r####"r"raw string" r#"raw "string""# r##"raw r#"string"#"## r###"raw r##"string"##"###"####);

    assert_next!(src, lexer, TOKEN, TokenKind::RawString, r#"r"raw string""#);
    assert_next!(src, lexer, TOKEN, TokenKind::RawString, r##"r#"raw "string""#"##);
    assert_next!(src, lexer, TOKEN, TokenKind::RawString, r###"r##"raw r#"string"#"##"###);
    assert_next!(src, lexer, TOKEN, TokenKind::RawString, r####"r###"raw r##"string"##"###"####);

    assert_next!(src, lexer, EOF);
}

#[test]
fn character() {
    init_lexer!(src, lexer, r#"'a' 'Z' '0' '\n' '\\'"#);

    assert_next!(src, lexer, TOKEN, TokenKind::Char, r#"'a'"#);
    assert_next!(src, lexer, TOKEN, TokenKind::Char, r#"'Z'"#);
    assert_next!(src, lexer, TOKEN, TokenKind::Char, r#"'0'"#);
    assert_next!(src, lexer, TOKEN, TokenKind::Char, r#"'\n'"#);
    assert_next!(src, lexer, TOKEN, TokenKind::Char, r#"'\\'"#);

    assert_next!(src, lexer, EOF);
}
