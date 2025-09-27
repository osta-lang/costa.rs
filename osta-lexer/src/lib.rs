pub mod lexer;
pub mod token;

pub use lexer::{Lexer, LexerError};
pub use token::{Token, TokenKind};

#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;
    use crate::token::{Token, TokenKind};

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
        "const static pub",
        kind @ TokenKind::Const,
        kind @ TokenKind::Static,
        kind @ TokenKind::Pub
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
        token @ Token { kind: TokenKind::Identifier, slice: "foo" },
        token @ Token { kind: TokenKind::Identifier, slice: "_foo" },
        token @ Token { kind: TokenKind::Identifier, slice: "foo123" },
        token @ Token { kind: TokenKind::Identifier, slice: "i32_" },
        token @ Token { kind: TokenKind::MacroIdentifier, slice: "@macro_name" },
        token @ Token { kind: TokenKind::ComptimeIdentifier, slice: "#comptime_name" },
        token @ Token { kind: TokenKind::DirectiveIdentifier, slice: "$directive_name" }
    );
    test_lex!(
        integers,
        r"
        123           1_2_3             1__2__3  _123    123_
        0b1010 0B1010 0b10_10 0B1_0_1_0 0b10__10 0B_1010 0b1010_
        0o7755 0O7755 0o77_55 0O7_7_5_5 0o77__55 0O_7755 0o7755_
        0xAA55 0XAA55 0xAA_55 0XA_A_5_5 0xAA__55 0X_AA55 0xAA55_
        ",
        token @ Token { kind: TokenKind::DecInt, slice: "123" },
        token @ Token { kind: TokenKind::DecInt, slice: "1_2_3" },
        token @ Token { kind: TokenKind::DecInt, slice: "1__2__3" },
        token @ Token { kind: TokenKind::Identifier, slice: "_123" },
        token @ Token { kind: TokenKind::DecInt, slice: "123" },
        token @ Token { kind: TokenKind::Identifier, slice: "_" },
        token @ Token { kind: TokenKind::BinInt, slice: "0b1010" },
        token @ Token { kind: TokenKind::BinInt, slice: "0B1010" },
        token @ Token { kind: TokenKind::BinInt, slice: "0b10_10" },
        token @ Token { kind: TokenKind::BinInt, slice: "0B1_0_1_0" },
        token @ Token { kind: TokenKind::BinInt, slice: "0b10__10" },
        token @ Token { kind: TokenKind::DecInt, slice: "0" },
        token @ Token { kind: TokenKind::Identifier, slice: "B_1010" },
        token @ Token { kind: TokenKind::BinInt, slice: "0b1010" },
        token @ Token { kind: TokenKind::Identifier, slice: "_" },
        token @ Token { kind: TokenKind::OctInt, slice: "0o7755" },
        token @ Token { kind: TokenKind::OctInt, slice: "0O7755" },
        token @ Token { kind: TokenKind::OctInt, slice: "0o77_55" },
        token @ Token { kind: TokenKind::OctInt, slice: "0O7_7_5_5" },
        token @ Token { kind: TokenKind::OctInt, slice: "0o77__55" },
        token @ Token { kind: TokenKind::DecInt, slice: "0" },
        token @ Token { kind: TokenKind::Identifier, slice: "O_7755" },
        token @ Token { kind: TokenKind::OctInt, slice: "0o7755" },
        token @ Token { kind: TokenKind::Identifier, slice: "_" },
        token @ Token { kind: TokenKind::HexInt, slice: "0xAA55" },
        token @ Token { kind: TokenKind::HexInt, slice: "0XAA55" },
        token @ Token { kind: TokenKind::HexInt, slice: "0xAA_55" },
        token @ Token { kind: TokenKind::HexInt, slice: "0XA_A_5_5" },
        token @ Token { kind: TokenKind::HexInt, slice: "0xAA__55" },
        token @ Token { kind: TokenKind::DecInt, slice: "0" },
        token @ Token { kind: TokenKind::Identifier, slice: "X_AA55" },
        token @ Token { kind: TokenKind::HexInt, slice: "0xAA55" },
        token @ Token { kind: TokenKind::Identifier, slice: "_" }
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
        token @ Token { kind: TokenKind::Float, slice: "1.0" },
        token @ Token { kind: TokenKind::IntFloat, slice: "1." },
        token @ Token { kind: TokenKind::FloatExp, slice: "1.0e10" },
        token @ Token { kind: TokenKind::FloatExp, slice: "1.0E10" },
        token @ Token { kind: TokenKind::FloatExp, slice: "1.0e-10" },
        token @ Token { kind: TokenKind::FloatExp, slice: "1.0E-10" },
        token @ Token { kind: TokenKind::FloatExp, slice: "1.0e+10" },
        token @ Token { kind: TokenKind::FloatExp, slice: "1.0E+10" },
        token @ Token { kind: TokenKind::Identifier, slice: "_1" },
        error => ".",
        token @ Token { kind: TokenKind::IntExp, slice: "0e10" },
        token @ Token { kind: TokenKind::DecInt, slice: "1" },
        token @ Token { kind: TokenKind::Identifier, slice: "_" },
        error => ".",
        token @ Token { kind: TokenKind::IntExp, slice: "0E10" },
        token @ Token { kind: TokenKind::IntFloat, slice: "1." },
        token @ Token { kind: TokenKind::Identifier, slice: "_0e" },
        error => "-",
        token @ Token { kind: TokenKind::DecInt, slice: "10" },
        token @ Token { kind: TokenKind::Float, slice: "1.0" },
        token @ Token { kind: TokenKind::Identifier, slice: "_E" },
        error => "-",
        token @ Token { kind: TokenKind::DecInt, slice: "10" },
        token @ Token { kind: TokenKind::Float, slice: "1.0" },
        token @ Token { kind: TokenKind::Identifier, slice: "e" },
        error => "+",
        token @ Token { kind: TokenKind::Identifier, slice: "_10" },
        token @ Token { kind: TokenKind::FloatExp, slice: "1.0E+10" },
        token @ Token { kind: TokenKind::Identifier, slice: "_" },
        token @ Token { kind: TokenKind::IntExp, slice: "1e10" },
        token @ Token { kind: TokenKind::IntExp, slice: "1E10" },
        token @ Token { kind: TokenKind::IntExp, slice: "1e-10" },
        token @ Token { kind: TokenKind::IntExp, slice: "1E-10" },
        token @ Token { kind: TokenKind::IntExp, slice: "1e+10" },
        token @ Token { kind: TokenKind::IntExp, slice: "1E+10" },
        token @ Token { kind: TokenKind::Identifier, slice: "_1e10" },
        token @ Token { kind: TokenKind::DecInt, slice: "1" },
        token @ Token { kind: TokenKind::Identifier, slice: "_E10" },
        token @ Token { kind: TokenKind::DecInt, slice: "1" },
        token @ Token { kind: TokenKind::Identifier, slice: "e" },
        error => "-",
        token @ Token { kind: TokenKind::Identifier, slice: "_10" },
        token @ Token { kind: TokenKind::IntExp, slice: "1E-1_0" },
        token @ Token { kind: TokenKind::IntExp, slice: "1e+10" },
        token @ Token { kind: TokenKind::Identifier, slice: "_" },
        token @ Token { kind: TokenKind::DecInt, slice: "1" },
        token @ Token { kind: TokenKind::Identifier, slice: "E_10" }
    );
    test_lex!(
        strings,
        r####"
        "this is a string" "this is a \"string\" with escapes" ""
        r#"this is a raw string"#
        r##"this is a raw string with "# in it"##
        r###"this is a raw string with ##" in it"###
        "####,
        token @ Token { kind: TokenKind::String, slice: r#""this is a string""# },
        token @ Token { kind: TokenKind::String, slice: r#""this is a \"string\" with escapes""# },
        token @ Token { kind: TokenKind::String, slice: r#""""# },
        token @ Token { kind: TokenKind::RawString, slice: r##"r#"this is a raw string"#"## },
        token @ Token { kind: TokenKind::RawString, slice: r###"r##"this is a raw string with "# in it"##"### },
        token @ Token { kind: TokenKind::RawString, slice: r####"r###"this is a raw string with ##" in it"###"#### }
    );
    test_lex!(
        open_string,
        r#""this is an open string"#,
        error,
        token @ Token { kind: TokenKind::Identifier, slice: "this" },
        token @ Token { kind: TokenKind::Identifier, slice: "is" },
        token @ Token { kind: TokenKind::Identifier, slice: "an" },
        token @ Token { kind: TokenKind::Identifier, slice: "open" },
        token @ Token { kind: TokenKind::Identifier, slice: "string" }
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
