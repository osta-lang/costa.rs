use crate::{err, ParseResult, ParseResultOpt, ParserError};
use osta_lexer::{Lexer, Token, TokenKind};
use osta_session::interner::InternId;
use osta_session::Session;
use osta_syntax::Span;

pub fn next(lexer: &mut Lexer) -> ParseResultOpt<Token> {
    match lexer.next() {
        Some(Ok(tok)) => Ok(Some(tok)),
        Some(Err(e)) => err!(ParserError::LexerError(e)),
        None => Ok(None),
    }
}

pub fn peek<'src>(lexer: &'src mut Lexer) -> ParseResultOpt<&'src Token> {
    match lexer.peek() {
        Some(Ok(tok)) => Ok(Some(tok)),
        Some(Err(e)) => err!(ParserError::LexerError(e.clone())),
        None => Ok(None),
    }
}

pub fn expect(lexer: &mut Lexer, kind: TokenKind) -> ParseResult<Token> {
    match next(lexer)? {
        Some(tok) if tok.kind == kind => Ok(tok),
        Some(tok) => {
            err!(ParserError::UnexpectedToken { found: tok.kind, expected: kind, span: tok.span })
        }
        None => err!(ParserError::UnexpectedEof),
    }
}

pub fn expect_opt<'src>(lexer: &'src mut Lexer, kind: TokenKind) -> ParseResultOpt<&'src Token> {
    match lexer.peek() {
        Some(Ok(tok)) if kind == tok.kind => Ok(Some(tok)),
        Some(Ok(_)) => Ok(None),
        Some(Err(e)) => err!(ParserError::LexerError(e.clone())),
        None => Ok(None),
    }
}

pub fn next_if(lexer: &mut Lexer, kind: TokenKind) -> ParseResultOpt<Token> {
    if expect_opt(lexer, kind)?.is_some() {
        next(lexer)
    } else {
        Ok(None)
    }
}

pub fn advance_if(lexer: &mut Lexer, kind: TokenKind) -> ParseResult<bool> {
    Ok(next_if(lexer, kind)?.is_some())
}

pub fn intern<'src>(session: &mut Session, lexer: &mut Lexer<'src>, span: &Span) -> InternId {
    let s = span.slice(lexer.source());
    session.get_or_intern(s)
}

#[macro_export]
macro_rules! expect_one_of {
    ($lexer: ident, $pat: pat) => {{
        let result: ParseResult<Token> = match $crate::util::next($lexer)? {
            Some(tok) if matches!(tok.kind, $pat) => Ok(tok),
            Some(tok) => err!(ParserError::CustomError {
                msg: format!("unexpected token: expected {:?}, found {:?}", stringify!($pat), tok),
                span: tok.span,
            }),
            None => err!(ParserError::UnexpectedEof),
        };
        result
    }};
}

#[macro_export]
macro_rules! try_parse {
    ($func: path, $session: ident, $lexer: ident, $builder: ident $(,$($tt: tt),+)?) => {{
        let checkpoint = $builder.checkpoint();
        let mut lexer_clone = $lexer.clone();
        let result = $func($session, &mut lexer_clone, $builder $(,$($tt),+)?);
        checkpoint.resolve(result).inspect(|_| *$lexer = lexer_clone)
    }};
}

#[macro_export]
macro_rules! choice {
    ($func: path, $session: ident, $lexer: ident, $builder: ident $(,$($args: expr),+)?; $($($tt: tt)+)?) => {
        try_parse!($func, $session, $lexer, $builder $(,$($args),+)?)$(.or_else(|err| {
            let result = choice!($($tt)*);
            $session.with_sta(|sta| err.merge(result, sta))
        }))?
    };
}
