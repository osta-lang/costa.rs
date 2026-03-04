use crate::{ParseResult, ParseResultOpt, ParserError};
use osta_lexer::{Lexer, Token, TokenKind};
use osta_session::interner::InternId;
use osta_session::Session;
use osta_syntax::Span;
use std::sync::{Arc, Mutex};

pub fn next(lexer: &mut Lexer) -> ParseResultOpt<Token> {
    match lexer.next() {
        Some(Ok(tok)) => Ok(Some(tok)),
        Some(Err(e)) => Err(ParserError::LexerError(e)),
        None => Ok(None),
    }
}

pub fn peek<'src>(lexer: &'src mut Lexer) -> ParseResultOpt<&'src Token> {
    match lexer.peek() {
        Some(Ok(tok)) => Ok(Some(tok)),
        Some(Err(e)) => Err(ParserError::LexerError(e.clone())),
        None => Ok(None),
    }
}

pub fn expect(lexer: &mut Lexer, kind: TokenKind) -> ParseResult<Token> {
    match next(lexer)? {
        Some(tok) if tok.kind == kind => Ok(tok),
        Some(tok) => {
            Err(ParserError::UnexpectedToken { found: tok.kind, expected: kind, span: tok.span })
        }
        None => Err(ParserError::UnexpectedEof),
    }
}

pub fn expect_opt<'src>(
    lexer: &'src mut Lexer,
    kind: TokenKind,
) -> ParseResultOpt<&'src Token> {
    match lexer.peek() {
        Some(Ok(tok)) if kind == tok.kind => Ok(Some(tok)),
        Some(Ok(_)) => Ok(None),
        Some(Err(e)) => Err(ParserError::LexerError(e.clone())),
        None => Ok(None),
    }
}

pub fn intern<'src>(
    session: Arc<Mutex<Session>>,
    lexer: &mut Lexer<'src>,
    span: &Span,
) -> InternId {
    let s = span.slice(lexer.source());
    let mut session = session.lock().unwrap();
    session.interner.get_or_intern(s)
}

pub fn intern_str(session: Arc<Mutex<Session>>, s: &str) -> InternId {
    let mut session = session.lock().unwrap();
    session.interner.get_or_intern(s)
}

#[macro_export]
macro_rules! try_parse {
    ($func: ident, $session: ident, $lexer: ident, $builder: ident $(,$($tt: tt),+)?) => {{
        let checkpoint = $builder.checkpoint();
        let mut lexer_clone = $lexer.clone();
        let result = $func($session.clone(), &mut lexer_clone, $builder $(,$($tt),+)?);
        checkpoint.resolve(result).inspect(|_| *$lexer = lexer_clone)
    }};
}
