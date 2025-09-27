pub mod lexer;
pub mod token;

pub use lexer::{Lexer, LexerError};
pub use token::{Token, TokenKind};

pub type Span = std::ops::Range<usize>;

#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;
    use crate::token::TokenKind;

    macro_rules! assert_lex_inner {
        ($lexer:ident, token, $pattern:pat) => {
            match $lexer.next() {
                Some(Ok($pattern)) => {}
                Some(Ok(token)) => panic!(
                    "unexpected token: {:?}\nexpected token: {}",
                    token,
                    stringify!($pattern)
                ),
                Some(Err(e)) => panic!("unexpected error: {:?}", e),
                None => panic!("unexpected end of input"),
            }
        };
        ($lexer:ident, kind, $pattern:pat) => {
            match $lexer.next() {
                Some(Ok(crate::token::Token { kind: $pattern, .. })) => {}
                Some(Ok(token)) => panic!(
                    "unexpected token kind: {:?}\nexpected kind: {}",
                    token.kind,
                    stringify!($pattern)
                ),
                Some(Err(e)) => panic!("unexpected error: {:?}", e),
                None => panic!("unexpected end of input"),
            }
        };
        ($lexer:ident, error, $pattern:pat) => {
            match $lexer.next() {
                Some(Err($pattern)) => {}
                Some(Err(e)) => panic!(
                    "unexpected error: {:?}\nexpecting error: {}",
                    e,
                    stringify!($pattern)
                ),
                Some(Ok(token)) => panic!(
                    "unexpected token: {:?}\nexpecting error: {}",
                    token,
                    stringify!($pattern)
                ),
                None => panic!("unexpected end of input"),
            }
        };
        ($lexer:ident, token) => {
            match $lexer.next() {
                Some(Ok(token)) => {}
                Some(Err(e)) => panic!("unexpected error: {:?}", e),
                None => panic!("unexpected end of input"),
            }
        };
        ($lexer:ident, error) => {
            match $lexer.next() {
                Some(Err(_)) => {}
                Some(Ok(token)) => panic!("unexpected token: {:?}\nexpecting any error!", token),
                None => panic!("unexpected end of input"),
            }
        };
    }

    macro_rules! assert_lex {
        ($lexer:ident, token $(,)?) => {
            assert_lex_inner!($lexer, token);
        };
        ($lexer:ident, error $(,)?) => {
            assert_lex_inner!($lexer, error);
        };
        ($lexer:ident, $kw:tt @ $kind:pat $(,)?) => {
            assert_lex_inner!($lexer, $kw, $kind);
        };
        ($lexer:ident, $kw:tt => $slice:expr $(,)?) => {
            assert_lex_inner!($lexer, $kw);
            assert_eq!($lexer.slice(), $slice);
        };
        ($lexer:ident, $kw:tt @ $kind:pat => $slice:expr $(,)?) => {
            assert_lex_inner!($lexer, $kw, $kind);
            assert_eq!($lexer.slice(), $slice);
        };
        ($lexer:ident, token, $($rest:tt)*) => {
            assert_lex!($lexer, token);
            assert_lex!($lexer, $($rest)*);
        };
        ($lexer:ident, error, $($rest:tt)*) => {
            assert_lex!($lexer, error);
            assert_lex!($lexer, $($rest)*);
        };
        ($lexer:ident, $kw:tt @ $kind:pat, $($rest:tt)*) => {
            assert_lex!($lexer, $kw @ $kind);
            assert_lex!($lexer, $($rest)*);
        };
        ($lexer:ident, $kw:tt => $slice:expr, $($rest:tt)*) => {
            assert_lex!($lexer, $kw => $slice);
            assert_lex!($lexer, $($rest)*);
        };
        ($lexer:ident, $kw:tt @ $kind:pat => $slice:expr, $($rest:tt)*) => {
            assert_lex!($lexer, $kw @ $kind => $slice);
            assert_lex!($lexer, $($rest)*);
        };
    }

    macro_rules! test_lex {
        ($name:ident, $input:expr, $($patterns:tt)*) => {
            #[test]
            fn $name() {
                let mut lexer = Lexer::new($input.trim());
                assert_lex!(lexer, $($patterns)*);
                assert_eq!(lexer.next(), None);
            }
        };
        ($name:ident, $input:expr) => {
            #[test]
            fn $name() {
                let mut lexer = Lexer::new($input.trim());
                assert_eq!(lexer.next(), None);
            }
        };
    }

    test_lex!(empty, "");
    test_lex!(
        comment,
        "// this is a comment",
        kind @ TokenKind::Comment
    );
    test_lex!(
        comment_block,
        r"/*
        this is a comment
        */",
        kind @ TokenKind::Comment
    );
    test_lex!(
        comment_block_nested,
        r"/*
        this is the start of a comment
        /* this is a comment inside a comment */
        this is the end of a comment
        */",
        kind @ TokenKind::Comment
    );
    test_lex!(
        keywords,
        r"
        const static pub move
        if else while for do break continue return match case
        where fn struct enum variant union type use mod impl trait extern
        let as
        ",
        kind @ TokenKind::Const,
        kind @ TokenKind::Static,
        kind @ TokenKind::Pub,
        kind @ TokenKind::Move,
        kind @ TokenKind::If,
        kind @ TokenKind::Else,
        kind @ TokenKind::While,
        kind @ TokenKind::For,
        kind @ TokenKind::Do,
        kind @ TokenKind::Break,
        kind @ TokenKind::Continue,
        kind @ TokenKind::Return,
        kind @ TokenKind::Match,
        kind @ TokenKind::Case,
        kind @ TokenKind::Where,
        kind @ TokenKind::Fn,
        kind @ TokenKind::Struct,
        kind @ TokenKind::Enum,
        kind @ TokenKind::Variant,
        kind @ TokenKind::Union,
        kind @ TokenKind::Type,
        kind @ TokenKind::Use,
        kind @ TokenKind::Mod,
        kind @ TokenKind::Impl,
        kind @ TokenKind::Trait,
        kind @ TokenKind::Extern,
        kind @ TokenKind::Let,
        kind @ TokenKind::As
    );
    test_lex!(
        primitives,
        "never void i1 i8 i16 i31 i32 i64 i128 isize u1 u8 u16 u31 u32 u64 u128 usize f16 f32 f64",
        kind @ TokenKind::Never,
        kind @ TokenKind::Void,
        kind @ TokenKind::IntType(1),
        kind @ TokenKind::IntType(8),
        kind @ TokenKind::IntType(16),
        kind @ TokenKind::IntType(31),
        kind @ TokenKind::IntType(32),
        kind @ TokenKind::IntType(64),
        kind @ TokenKind::IntType(128),
        kind @ TokenKind::IsizeType,
        kind @ TokenKind::UintType(1),
        kind @ TokenKind::UintType(8),
        kind @ TokenKind::UintType(16),
        kind @ TokenKind::UintType(31),
        kind @ TokenKind::UintType(32),
        kind @ TokenKind::UintType(64),
        kind @ TokenKind::UintType(128),
        kind @ TokenKind::UsizeType,
        kind @ TokenKind::FloatType(16),
        kind @ TokenKind::FloatType(32),
        kind @ TokenKind::FloatType(64)
    );
    test_lex!(
        identifiers,
        "foo _foo foo123 i32_ @macro_name #comptime_name $directive_name",
        kind @ TokenKind::Identifier => "foo",
        kind @ TokenKind::Identifier => "_foo",
        kind @ TokenKind::Identifier => "foo123",
        kind @ TokenKind::Identifier => "i32_",
        kind @ TokenKind::MacroIdentifier => "@macro_name",
        kind @ TokenKind::ComptimeIdentifier => "#comptime_name",
        kind @ TokenKind::DirectiveIdentifier => "$directive_name"
    );
    test_lex!(
        integers,
        r"
        123           1_2_3             1__2__3  _123    123_
        0b1010 0B1010 0b10_10 0B1_0_1_0 0b10__10 0B_1010 0b1010_
        0o7755 0O7755 0o77_55 0O7_7_5_5 0o77__55 0O_7755 0o7755_
        0xAA55 0XAA55 0xAA_55 0XA_A_5_5 0xAA__55 0X_AA55 0xAA55_
        ",
        kind @ TokenKind::DecInt => "123",
        kind @ TokenKind::DecInt => "1_2_3",
        kind @ TokenKind::DecInt => "1__2__3",
        kind @ TokenKind::Identifier => "_123",
        kind @ TokenKind::DecInt => "123",
        kind @ TokenKind::Identifier => "_",
        kind @ TokenKind::BinInt => "0b1010",
        kind @ TokenKind::BinInt => "0B1010",
        kind @ TokenKind::BinInt => "0b10_10",
        kind @ TokenKind::BinInt => "0B1_0_1_0",
        kind @ TokenKind::BinInt => "0b10__10",
        kind @ TokenKind::DecInt => "0",
        kind @ TokenKind::Identifier => "B_1010",
        kind @ TokenKind::BinInt => "0b1010",
        kind @ TokenKind::Identifier => "_",
        kind @ TokenKind::OctInt => "0o7755",
        kind @ TokenKind::OctInt => "0O7755",
        kind @ TokenKind::OctInt => "0o77_55",
        kind @ TokenKind::OctInt => "0O7_7_5_5",
        kind @ TokenKind::OctInt => "0o77__55",
        kind @ TokenKind::DecInt => "0",
        kind @ TokenKind::Identifier => "O_7755",
        kind @ TokenKind::OctInt => "0o7755",
        kind @ TokenKind::Identifier => "_",
        kind @ TokenKind::HexInt => "0xAA55",
        kind @ TokenKind::HexInt => "0XAA55",
        kind @ TokenKind::HexInt => "0xAA_55",
        kind @ TokenKind::HexInt => "0XA_A_5_5",
        kind @ TokenKind::HexInt => "0xAA__55",
        kind @ TokenKind::DecInt => "0",
        kind @ TokenKind::Identifier => "X_AA55",
        kind @ TokenKind::HexInt => "0xAA55",
        kind @ TokenKind::Identifier => "_"
    );
    test_lex!(
        floats,
        r"
        1.0 1.
        1.0e10  1.0E10  1.0e-10  1.0E-10  1.0e+10  1.0E+10
        _1.0e10 1_.0E10 1._0e-10 1.0_E-10 1.0e+_10 1.0E+10_
        1e10    1E10    1e-10    1E-10    1e+10    1E+10
        _1e10   1_E10   1e-_10   1E-1_0   1e+10_   1E_10
        ",
        kind @ TokenKind::Float => "1.0",
        kind @ TokenKind::IntFloat => "1.",
        kind @ TokenKind::FloatExp => "1.0e10",
        kind @ TokenKind::FloatExp => "1.0E10",
        kind @ TokenKind::FloatExp => "1.0e-10",
        kind @ TokenKind::FloatExp => "1.0E-10",
        kind @ TokenKind::FloatExp => "1.0e+10",
        kind @ TokenKind::FloatExp => "1.0E+10",
        kind @ TokenKind::Identifier => "_1",
        error => ".",
        kind @ TokenKind::IntExp => "0e10",
        kind @ TokenKind::DecInt => "1",
        kind @ TokenKind::Identifier => "_",
        error => ".",
        kind @ TokenKind::IntExp => "0E10",
        kind @ TokenKind::IntFloat => "1.",
        kind @ TokenKind::Identifier => "_0e",
        error => "-",
        kind @ TokenKind::DecInt => "10",
        kind @ TokenKind::Float => "1.0",
        kind @ TokenKind::Identifier => "_E",
        error => "-",
        kind @ TokenKind::DecInt => "10",
        kind @ TokenKind::Float => "1.0",
        kind @ TokenKind::Identifier => "e",
        error => "+",
        kind @ TokenKind::Identifier => "_10",
        kind @ TokenKind::FloatExp => "1.0E+10",
        kind @ TokenKind::Identifier => "_",
        kind @ TokenKind::IntExp => "1e10",
        kind @ TokenKind::IntExp => "1E10",
        kind @ TokenKind::IntExp => "1e-10",
        kind @ TokenKind::IntExp => "1E-10",
        kind @ TokenKind::IntExp => "1e+10",
        kind @ TokenKind::IntExp => "1E+10",
        kind @ TokenKind::Identifier => "_1e10",
        kind @ TokenKind::DecInt => "1",
        kind @ TokenKind::Identifier => "_E10",
        kind @ TokenKind::DecInt => "1",
        kind @ TokenKind::Identifier => "e",
        error => "-",
        kind @ TokenKind::Identifier => "_10",
        kind @ TokenKind::IntExp => "1E-1_0",
        kind @ TokenKind::IntExp => "1e+10",
        kind @ TokenKind::Identifier => "_",
        kind @ TokenKind::DecInt => "1",
        kind @ TokenKind::Identifier => "E_10"
    );
    test_lex!(
        strings,
        r####"
        "this is a string" "this is a \"string\" with escapes" ""
        r#"this is a raw string"#
        r##"this is a raw string with "# in it"##
        r###"this is a raw string with ##" in it"###
        "####,
        kind @ TokenKind::String => r#""this is a string""#,
        kind @ TokenKind::String => r#""this is a \"string\" with escapes""#,
        kind @ TokenKind::String => r#""""#,
        kind @ TokenKind::RawString => r##"r#"this is a raw string"#"##,
        kind @ TokenKind::RawString => r###"r##"this is a raw string with "# in it"##"###,
        kind @ TokenKind::RawString => r####"r###"this is a raw string with ##" in it"###"####
    );
    test_lex!(
        open_string,
        r#""this is an open string"#,
        error,
        kind @ TokenKind::Identifier => "this",
        kind @ TokenKind::Identifier => "is",
        kind @ TokenKind::Identifier => "an",
        kind @ TokenKind::Identifier => "open",
        kind @ TokenKind::Identifier => "string"
    );
    test_lex!(
        open_raw_string,
        r##"r#"this is a raw string without end""##,
        error
    );
    test_lex!(
        symbols,
        r"
        ( ) { } [ ]
        , : ;
        ->
        ",
        kind @ TokenKind::LParen => "(",
        kind @ TokenKind::RParen => ")",
        kind @ TokenKind::LBrace => "{",
        kind @ TokenKind::RBrace => "}",
        kind @ TokenKind::LBracket => "[",
        kind @ TokenKind::RBracket => "]",
        kind @ TokenKind::Comma => ",",
        kind @ TokenKind::Colon => ":",
        kind @ TokenKind::Semicolon => ";",
        kind @ TokenKind::Arrow => "->"
    );
}
