use crate::{LexerError, Span};
use logos::Logos;

#[derive(Debug, PartialEq)]
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
#[logos(error = LexerError)]
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
    #[token("/*", lex_block_comment)]
    #[token("//", lex_line_comment)]
    Comment,
    Operator(usize),
    // ========
    // Keywords
    // ========
    // Modifiers
    #[token("const")]
    Const,
    #[token("static")]
    Static,
    #[token("pub")]
    Pub,
    #[token("move")]
    Move,
    // Control flow
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
    // Variables
    #[token("let")]
    Let,
    #[token("as")]
    As,
    // Data types
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
    // =====
    // Atoms
    // =====
    // Identifiers
    #[regex(r"(?&ident)")]
    Identifier,
    #[regex(r"@(?&ident)")]
    MacroIdentifier,
    #[regex(r"#(?&ident)")]
    ComptimeIdentifier,
    #[regex(r"\$(?&ident)")]
    DirectiveIdentifier,
    // Integers
    #[regex("(?&dec_int)")]
    DecInt,
    #[regex(r"0[bB][01]+(_+[01]+)*")]
    BinInt,
    #[regex(r"0[oO][0-7]+(_+[0-7]+)*")]
    OctInt,
    #[regex(r"0[xX][0-9a-fA-F]+(_+[0-9a-fA-F]+)*")]
    HexInt,
    // Floats
    #[regex(r"(?&dec_int)\.(?&dec_int)")]
    Float,
    #[regex(r"(?&dec_int)\.")]
    IntFloat,
    #[regex(r"(?&dec_int)\.(?&dec_int)[eE][+-]?(?&dec_int)")]
    FloatExp,
    #[regex(r"(?&dec_int)[eE][+-]?(?&dec_int)")]
    IntExp,
    // Strings
    #[regex(r#""(?:[^"]|\\")*""#)]
    String,
    #[regex(r#"r#*""#, lex_raw_string)]
    RawString,
    // =======
    // Symbols
    // =======
    // Enclosures
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    // Punctuation
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token(";")]
    Semicolon,
    #[token("->")]
    Arrow,
}

fn lex_nty(lexer: &mut logos::Lexer<TokenKind>) -> Result<usize, LexerError> {
    let slice = lexer.slice();
    let nty = slice[1..].parse::<usize>()?;
    Ok(nty)
}

fn lex_raw_string(lexer: &mut logos::Lexer<TokenKind>) -> bool {
    let hashes = lexer.slice().len() - 2;
    let mut exiting = false;
    let mut escape = false;
    let mut count = hashes;

    for c in lexer.remainder().chars() {
        lexer.bump(1);

        if exiting {
            if c == '#' {
                if count == 1 {
                    return true;
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
            exiting = true;
        } else {
            escape = false;
        }
    }

    false
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

fn lex_block_comment(lexer: &mut logos::Lexer<TokenKind>) -> Result<(), LexerError> {
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

    Err(LexerError::UnterminatedBlockComment)
}
