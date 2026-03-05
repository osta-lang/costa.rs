#[macro_use]
mod common;

use osta_lexer::{Lexer, TokenKind};

#[test]
fn self_type() {
    init_lexer!(src, lexer, "Self self");
    assert_next!(src, lexer, TOKEN, TokenKind::SelfType, "Self");
    assert_next!(src, lexer, TOKEN, TokenKind::SelfValue, "self");
    assert_next!(src, lexer, EOF);
}

#[test]
fn path_resolution() {
    init_lexer!(src, lexer, "package super");

    assert_next!(src, lexer, TOKEN, TokenKind::Package, "package");
    assert_next!(src, lexer, TOKEN, TokenKind::Super, "super");

    assert_next!(src, lexer, EOF);
}

#[test]
fn modifiers() {
    init_lexer!(src, lexer, "const static let pub move");

    // storage
    assert_next!(src, lexer, TOKEN, TokenKind::Const, "const");
    assert_next!(src, lexer, TOKEN, TokenKind::Static, "static");
    assert_next!(src, lexer, TOKEN, TokenKind::Let, "let");
    // visibility
    assert_next!(src, lexer, TOKEN, TokenKind::Pub, "pub");
    // ownership
    assert_next!(src, lexer, TOKEN, TokenKind::Move, "move");

    assert_next!(src, lexer, EOF);
}

#[test]
fn control_flow() {
    init_lexer!(src, lexer, "if else while for do break continue return");

    assert_next!(src, lexer, TOKEN, TokenKind::If, "if");
    assert_next!(src, lexer, TOKEN, TokenKind::Else, "else");
    assert_next!(src, lexer, TOKEN, TokenKind::While, "while");
    assert_next!(src, lexer, TOKEN, TokenKind::For, "for");
    assert_next!(src, lexer, TOKEN, TokenKind::Do, "do");
    assert_next!(src, lexer, TOKEN, TokenKind::Break, "break");
    assert_next!(src, lexer, TOKEN, TokenKind::Continue, "continue");
    assert_next!(src, lexer, TOKEN, TokenKind::Return, "return");

    assert_next!(src, lexer, EOF);
}

#[test]
fn pattern_matching() {
    init_lexer!(src, lexer, "match case");

    assert_next!(src, lexer, TOKEN, TokenKind::Match, "match");
    assert_next!(src, lexer, TOKEN, TokenKind::Case, "case");

    assert_next!(src, lexer, EOF);
}

#[test]
fn declarations() {
    init_lexer!(src, lexer, "where fn struct enum variant union type use mod impl trait extern");

    assert_next!(src, lexer, TOKEN, TokenKind::Where, "where");
    assert_next!(src, lexer, TOKEN, TokenKind::Fn, "fn");
    assert_next!(src, lexer, TOKEN, TokenKind::Struct, "struct");
    assert_next!(src, lexer, TOKEN, TokenKind::Enum, "enum");
    assert_next!(src, lexer, TOKEN, TokenKind::Variant, "variant");
    assert_next!(src, lexer, TOKEN, TokenKind::Union, "union");
    assert_next!(src, lexer, TOKEN, TokenKind::Type, "type");
    assert_next!(src, lexer, TOKEN, TokenKind::Use, "use");
    assert_next!(src, lexer, TOKEN, TokenKind::Mod, "mod");
    assert_next!(src, lexer, TOKEN, TokenKind::Impl, "impl");
    assert_next!(src, lexer, TOKEN, TokenKind::Trait, "trait");
    assert_next!(src, lexer, TOKEN, TokenKind::Extern, "extern");

    assert_next!(src, lexer, EOF);
}

#[test]
fn type_casting() {
    init_lexer!(src, lexer, "as");

    assert_next!(src, lexer, TOKEN, TokenKind::As, "as");

    assert_next!(src, lexer, EOF);
}

#[test]
fn primitives() {
    init_lexer!(
        src,
        lexer,
        "never void u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize f32 f64"
    );

    assert_next!(src, lexer, TOKEN, TokenKind::Never, "never");
    assert_next!(src, lexer, TOKEN, TokenKind::Void, "void");
    assert_next!(src, lexer, TOKEN, TokenKind::UintType(8), "u8");
    assert_next!(src, lexer, TOKEN, TokenKind::UintType(16), "u16");
    assert_next!(src, lexer, TOKEN, TokenKind::UintType(32), "u32");
    assert_next!(src, lexer, TOKEN, TokenKind::UintType(64), "u64");
    assert_next!(src, lexer, TOKEN, TokenKind::UintType(128), "u128");
    assert_next!(src, lexer, TOKEN, TokenKind::UsizeType, "usize");
    assert_next!(src, lexer, TOKEN, TokenKind::IntType(8), "i8");
    assert_next!(src, lexer, TOKEN, TokenKind::IntType(16), "i16");
    assert_next!(src, lexer, TOKEN, TokenKind::IntType(32), "i32");
    assert_next!(src, lexer, TOKEN, TokenKind::IntType(64), "i64");
    assert_next!(src, lexer, TOKEN, TokenKind::IntType(128), "i128");
    assert_next!(src, lexer, TOKEN, TokenKind::IsizeType, "isize");
    assert_next!(src, lexer, TOKEN, TokenKind::FloatType(32), "f32");
    assert_next!(src, lexer, TOKEN, TokenKind::FloatType(64), "f64");

    assert_next!(src, lexer, EOF);
}
