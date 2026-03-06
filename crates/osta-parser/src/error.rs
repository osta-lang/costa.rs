use osta_alloc::{alloc_vec, AllocVec, ErasedAllocator};
use osta_ast::NodeId;
use osta_lexer::TokenKind;
use osta_session::ShortTermAllocator;
use osta_syntax::Span;
use std::fmt::{Display, Formatter};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParserError {
    #[error(transparent)]
    LexerError(#[from] osta_lexer::LexerError),
    #[error("invalid type: expected a type, found {found:?}")]
    InvalidType { found: TokenKind, span: Span },
    #[error(
        "invalid path: expected an identifier, 'self', 'super', or 'package', found {found:?}"
    )]
    InvalidPath { found: TokenKind, span: Span },
    #[error("invalid prefix operator: expected '-', '*', '!', or '~', found {found:?}")]
    InvalidPrefixOperator { found: TokenKind, span: Span },
    #[error("ambiguous operator at the same precedence level")]
    AmbiguousOperator { span: Span },
    #[error("unexpected token: expected {expected:?}, found {found:?}")]
    UnexpectedToken { expected: TokenKind, found: TokenKind, span: Span },
    #[error("unexpected EOF")]
    UnexpectedEof,
    #[error("{msg}")]
    CustomError { msg: String, span: Span },
}

#[derive(Debug, Error)]
pub struct ErrorList(AllocVec<ParserError, ErasedAllocator<ShortTermAllocator>>);

impl ErrorList {
    pub fn pair(a: ParserError, b: ParserError, allocator: &ShortTermAllocator) -> Self {
        ErrorList(alloc_vec!(allocator, [a, b]))
    }

    pub fn push(mut self, error: ParserError) -> Self {
        self.0.push(error);
        self
    }

    pub fn merge(mut self, mut other: Self) -> Self {
        if self.0.len() >= other.0.len() {
            self.0.append(&mut other.0);
            self
        } else {
            other.0.append(&mut self.0);
            other
        }
    }
}

impl Display for ErrorList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Multiple errors:")?;
        for err in &self.0 {
            writeln!(f, "====================")?;
            writeln!(f, "{err}")?;
            writeln!(f, "====================\n")?;
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ErrorBundle {
    #[error(transparent)]
    Single(#[from] ParserError),
    #[error(transparent)]
    List(#[from] ErrorList),
}

impl ErrorBundle {
    pub fn merge<T>(
        self,
        result: ParseResult<T>,
        allocator: &ShortTermAllocator,
    ) -> ParseResult<T> {
        match result {
            Ok(t) => Ok(t),
            Err(err) => Err(self.join(err, allocator)),
        }
    }

    fn join(self, other: Self, allocator: &ShortTermAllocator) -> Self {
        match self {
            ErrorBundle::Single(err) => match other {
                ErrorBundle::Single(other) => {
                    ErrorBundle::List(ErrorList::pair(err, other, allocator))
                }
                ErrorBundle::List(others) => ErrorBundle::List(others.push(err)),
            },
            ErrorBundle::List(errs) => match other {
                ErrorBundle::Single(other) => ErrorBundle::List(errs.push(other)),
                ErrorBundle::List(others) => ErrorBundle::List(errs.merge(others)),
            },
        }
    }
}

pub type ParseResult<T = (NodeId, Span)> = Result<T, ErrorBundle>;
pub type ParseResultOpt<T = (NodeId, Span)> = ParseResult<Option<T>>;

#[macro_export]
macro_rules! err {
    ($expr: expr) => {
        Err($expr.into())
    };
}
