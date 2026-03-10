use crate::{err, FileSession, ParseResult, ParseResultOpt, ParserErrorKind};
use osta_lexer::{Token, TokenKind};

pub fn next() -> ParseResultOpt<Token> {
    let lexer = FileSession::lexer();
    match lexer.next() {
        Some(Ok(tok)) => Ok(Some(tok)),
        Some(Err(e)) => err!(ParserErrorKind::LexerError(e)),
        None => Ok(None),
    }
}

pub fn peek() -> ParseResultOpt<&'static Token> {
    let lexer = FileSession::lexer();
    match lexer.peek() {
        Some(Ok(tok)) => Ok(Some(tok)),
        Some(Err(e)) => err!(ParserErrorKind::LexerError(e.clone())),
        None => Ok(None),
    }
}

pub fn expect(kind: TokenKind) -> ParseResult<Token> {
    match next()? {
        Some(tok) if tok.kind == kind => Ok(tok),
        Some(tok) => {
            err!(ParserErrorKind::ExpectedToken { found: tok.kind, expected: kind, span: tok.span })
        }
        None => err!(ParserErrorKind::UnexpectedEof),
    }
}

pub fn expect_opt<'src>(kind: TokenKind) -> ParseResultOpt<&'src Token> {
    let lexer = FileSession::lexer();
    match lexer.peek() {
        Some(Ok(tok)) if kind == tok.kind => Ok(Some(tok)),
        Some(Ok(_)) => Ok(None),
        Some(Err(e)) => err!(ParserErrorKind::LexerError(e.clone())),
        None => Ok(None),
    }
}

pub fn next_if(kind: TokenKind) -> ParseResultOpt<Token> {
    if expect_opt(kind)?.is_some() {
        next()
    } else {
        Ok(None)
    }
}

pub fn advance_if(kind: TokenKind) -> ParseResult<bool> {
    Ok(next_if(kind)?.is_some())
}

#[inline(always)]
pub fn unsafe_next() -> Token {
    let lexer = FileSession::lexer();
    unsafe { lexer.next().unwrap_unchecked().unwrap_unchecked() }
}

#[macro_export]
macro_rules! expect_choice {
    ($pat: pat) => {{
        let result: ParseResult<Token> = match $crate::util::next()? {
            Some(tok) if matches!(tok.kind, $pat) => Ok(tok),
            Some(tok) => err!(ParserErrorKind::CustomError {
                code: Some("parser::expected_token"),
                msg: format!("unexpected token: expected {:?}, found {:?}", stringify!($pat), tok),
                span: tok.span,
            }),
            None => err!(ParserErrorKind::UnexpectedEof),
        };
        result
    }};
}

pub(crate) struct Checkpoint(osta_ast::builder::Checkpoint);

impl Checkpoint {
    pub fn new() -> Self {
        FileSession::lexer().checkpoint();
        let builder = FileSession::ast();
        let checkpoint = builder.checkpoint();
        Self(checkpoint)
    }

    pub fn commit(self) {
        FileSession::lexer().commit();
        self.0.commit();
    }

    pub fn rollback(self) {
        FileSession::lexer().rollback();
        self.0.rollback();
    }

    pub fn resolve<T, E>(self, result: Result<T, E>) -> Result<T, E> {
        match result {
            Ok(ok) => {
                self.commit();
                Ok(ok)
            }
            Err(err) => {
                self.rollback();
                Err(err)
            }
        }
    }
}

#[macro_export]
macro_rules! try_parse {
    ($func: path $(,$($tt: tt),+)?) => {
        $crate::util::Checkpoint::new().resolve($func($($($tt),+)?))
    };
}

#[macro_export]
macro_rules! choice {
    ($func: path, $session: ident, $lexer: ident, $builder: ident $(,$($args: expr),+)?; $($($tt: tt)+)?) => {
        try_parse!($func, $session, $lexer, $builder $(,$($args),+)?)$(.or_else(|err| {
            match choice!($($tt)*) {
                Ok(result) => Ok(result),
                Err(e) => Err(::osta_diagnostic::RequireSolvingOne::new(vec![err, e]).into()),
            }
        }))?
    };
}
