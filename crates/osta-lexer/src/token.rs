use crate::lexer::LexerErrorKind;
use logos::Logos;
use osta_syntax::Span;

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Logos, Clone, Debug, PartialEq)]
#[logos(error = LexerErrorKind)]
#[logos(skip r"[ \t\r\n\f]+")]
#[logos(subpattern dec_int = r"[0-9]+(_+[0-9]+)*")]
#[logos(subpattern pos_int = r"0*[1-9][0-9]*")]
#[cfg_attr(
    feature = "unicode-identifiers",
    logos(subpattern ident = r"(\p{XID_Start}|_)\p{XID_Continue}*")
)]
#[cfg_attr(
    not(feature = "unicode-identifiers"),
    logos(subpattern ident = r"[a-zA-Z_][a-zA-Z0-9_]*")
)]
pub enum TokenKind {
    #[token("//", lex_line_comment)]
    #[token("/*", lex_block_comment)]
    Comment,

    // ========
    // Keywords
    // ========

    // Self References
    #[token("Self")]
    SelfType,
    #[token("self")]
    SelfValue,

    // Path Resolution
    #[token("super")]
    Super,
    #[token("package")]
    Package,

    // Storage Declarations
    #[token("const")]
    Const,
    #[token("static")]
    Static,
    #[token("let")]
    Let,

    // Visibility Modifiers
    #[token("pub")]
    Pub,

    // Ownership Modifiers
    #[token("move")]
    Move,

    // Control Flow
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("while")]
    While,
    #[token("for")]
    For,
    #[token("do")]
    Do,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("return")]
    Return,

    // Pattern Matching
    #[token("match")]
    Match,
    #[token("case")]
    Case,

    // Declarations
    #[token("where")]
    Where,
    #[token("fn")]
    Fn,
    #[token("struct")]
    Struct,
    #[token("enum")]
    Enum,
    #[token("variant")]
    Variant,
    #[token("union")]
    Union,
    #[token("type")]
    Type,
    #[token("use")]
    Use,
    #[token("mod")]
    Mod,
    #[token("impl")]
    Impl,
    #[token("trait")]
    Trait,
    #[token("extern")]
    Extern,

    // Type Casting
    #[token("as")]
    As,

    // Built-in Types
    #[token("never")]
    Never,
    #[token("void")]
    Void,
    #[regex("u(?&pos_int)", lex_nty)]
    UintType(usize),
    #[token("usize")]
    UsizeType,
    #[regex("i(?&pos_int)", lex_nty)]
    IntType(usize),
    #[token("isize")]
    IsizeType,
    #[regex("f(?&pos_int)", lex_nty)]
    FloatType(usize),

    // ===========
    // Identifiers
    // ===========

    // Regular
    #[regex(r"(?&ident)")]
    Identifier,

    // Macro (@-prefixed)
    #[regex(r"@(?&ident)")]
    MacroIdentifier,

    // Comptime (#-prefixed)
    #[regex(r"#(?&ident)")]
    ComptimeIdentifier,

    // Directive ($-prefixed)
    #[regex(r"\$(?&ident)")]
    DirectiveIdentifier,

    // Lifetime
    #[regex(r"'(?&ident)")]
    LifetimeIdentifier,

    // ========
    // Literals
    // ========

    // Integer (Decimal)
    #[regex("(?&dec_int)")]
    DecInt,

    // Integer (Binary)
    #[regex(r"0[bB][01]+(_+[01]+)*")]
    BinInt,

    // Integer (Octal)
    #[regex(r"0[oO][0-7]+(_+[0-7]+)*")]
    OctInt,

    // Integer (Hexadecimal)
    #[regex(r"0[xX][0-9a-fA-F]+(_+[0-9a-fA-F]+)*")]
    HexInt,

    // Float (Decimal)
    #[regex(r"(?&dec_int)\.(?&dec_int)")]
    Float,

    // Float (Integer part only)
    #[regex(r"(?&dec_int)\.")]
    IntFloat,

    // Float (With Exponent)
    #[regex(r"(?&dec_int)\.(?&dec_int)[eE][+-]?(?&dec_int)")]
    FloatExp,

    // Float (Integer part only, with Exponent)
    #[regex(r"(?&dec_int)[eE][+-]?(?&dec_int)")]
    IntExp,

    // String (Escaped)
    #[regex(r#""(?:[^"]|\\")*""#)]
    String,

    // String (Raw)
    #[regex(r#"r#*""#, lex_raw_string)]
    RawString,

    // Character
    #[regex(r#"'([^'\\]|\\.)'"#)]
    Char,

    // =========
    // Operators
    // =========

    // Arithmetic
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,

    // Bitwise
    #[token("&")]
    Ampersand,
    #[token("|")]
    Pipe,
    #[token("^")]
    Caret,
    #[token("<<")]
    LShift,
    #[token(">>")]
    RShift,
    #[token(">>>")]
    ARShift,
    #[token("~")]
    Tilde,

    // Assignment
    #[token("=")]
    Equal,
    #[token("+=")]
    PlusEqual,
    #[token("-=")]
    MinusEqual,
    #[token("*=")]
    StarEqual,
    #[token("/=")]
    SlashEqual,
    #[token("%=")]
    PercentEqual,
    #[token("&=")]
    AmpersandEqual,
    #[token("|=")]
    PipeEqual,
    #[token("^=")]
    CaretEqual,
    #[token("<<=")]
    LShiftEqual,
    #[token(">>=")]
    RShiftEqual,
    #[token(">>>=")]
    ARShiftEqual,

    // Comparison
    #[token("==")]
    DoubleEqual,
    #[token("!=")]
    NotEqual,
    #[token("<")]
    Less,
    #[token("<=")]
    LessEqual,
    #[token(">")]
    Greater,
    #[token(">=")]
    GreaterEqual,

    // Logical
    #[token("&&")]
    DoubleAmpersand,
    #[token("||")]
    DoublePipe,

    // Range
    #[token("..")]
    DoubleDot,
    #[token("..=")]
    DoubleDotEqual,

    // Unary
    #[token("++")]
    DoublePlus,
    #[token("--")]
    DoubleMinus,
    #[token("!")]
    Bang,
    #[token("?")]
    Question,

    // ===========
    // Punctuation
    // ===========

    // Grouping (Parentheses)
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,

    // Grouping (Braces)
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,

    // Grouping (Brackets)
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,

    // Access/Separation
    #[token(".")]
    Dot,

    // Path/Qualification
    #[token("::")]
    DoubleColon,

    // Separator
    #[token(",")]
    Comma,

    // Label/Type Annotation
    #[token(":")]
    Colon,

    // Statement Terminator
    #[token(";")]
    Semicolon,

    // Function Return Type
    #[token("->")]
    Arrow,

    // Variadic
    #[token("...")]
    Ellipsis,

    // Attributes
    #[token("#")]
    Pound,
}

fn lex_line_comment(lexer: &mut logos::Lexer<TokenKind>) -> bool {
    for c in lexer.remainder().chars() {
        if c == '\r' || c == '\n' {
            return true;
        }
        lexer.bump(1);
    }
    true
}

fn lex_block_comment(lexer: &mut logos::Lexer<TokenKind>) -> Result<(), LexerErrorKind> {
    let mut depth = 1;
    let mut prev = '\0';

    for c in lexer.remainder().chars() {
        lexer.bump(1);

        if prev == '*' && c == '/' {
            depth -= 1;
            if depth == 0 {
                return Ok(());
            }
        }

        if prev == '/' && c == '*' {
            depth += 1;
        }

        prev = c;
    }

    Err(LexerErrorKind::UnterminatedBlockComment)
}

fn lex_nty(lexer: &mut logos::Lexer<TokenKind>) -> Result<usize, LexerErrorKind> {
    let slice = lexer.slice();
    let nty = slice[1..].parse::<usize>()?;
    Ok(nty)
}

fn lex_raw_string(lexer: &mut logos::Lexer<TokenKind>) -> Result<(), LexerErrorKind> {
    let hashes = lexer.slice().len() - 2;
    let mut exiting = false;
    let mut escape = false;
    let mut count = hashes;

    for c in lexer.remainder().chars() {
        lexer.bump(1);

        if exiting {
            if c == '#' {
                if count == 1 {
                    return Ok(());
                }

                count -= 1;

                continue;
            } else {
                count = hashes;
                exiting = false;
            }
        }

        if c == '\\' {
            escape = !escape;
        } else if c == '"' && !escape {
            if count == 0 {
                return Ok(());
            }

            exiting = true;
        } else {
            escape = false;
        }
    }

    Err(LexerErrorKind::UnterminatedString)
}
