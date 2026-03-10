use osta_ast::NodeId;
use osta_diagnostic::{Diagnostic, DiagnosticLabeledSpan, Severity};
use osta_lexer::{LexerError, TokenKind};
use osta_syntax::Span;
use std::cell::RefCell;
use std::fmt::Display;
use thiserror::Error;

thread_local! {
    static INTENT_STACK: RefCell<Vec<ParserIntent>> = RefCell::new(Vec::new());
}

pub(crate) fn push_intent(intent: ParserIntent) {
    INTENT_STACK.with_borrow_mut(|stack| stack.push(intent));
}

pub(crate) fn pop_intent() {
    INTENT_STACK.with_borrow_mut(|stack| stack.pop().unwrap());
}

#[macro_export]
macro_rules! scoped_intent {
    ($intent:expr) => {
        $crate::error::push_intent($intent);
        let _deferred_scoped_intent = ::defer_rs::Defer::new(|| $crate::error::pop_intent());
    };
}

#[derive(Debug, Error)]
pub enum ParserErrorKind {
    #[error(transparent)]
    LexerError(#[from] LexerError),
    #[error("unexpected token: fount {found:?}")]
    UnexpectedToken { found: TokenKind, span: Span },
    #[error("ambiguous operator at the same precedence level")]
    AmbiguousOperator { span: Span },
    #[error("unexpected token: expected {expected:?}, found {found:?}")]
    ExpectedToken { expected: TokenKind, found: TokenKind, span: Span },
    #[error("unexpected EOF")]
    UnexpectedEof,
    #[error("{msg}")]
    CustomError { code: Option<&'static str>, msg: String, span: Span },
}

#[derive(Debug, Clone)]
pub enum ParserIntent {
    TopLevel,
    FuncDecl,
    Block,
    Statement,
    Expression,
}

#[derive(Debug, Error)]
#[error("{intent_stack:?}: {kind}")]
pub struct ParserError {
    kind: ParserErrorKind,
    intent_stack: Vec<ParserIntent>,
}

impl ParserError {
    pub fn new(kind: ParserErrorKind, intent_stack: Vec<ParserIntent>) -> Self {
        Self { kind, intent_stack }
    }
}

impl From<ParserErrorKind> for ParserError {
    fn from(kind: ParserErrorKind) -> Self {
        let intent_stack = INTENT_STACK.with_borrow_mut(|stack| stack.clone());
        Self::new(kind, intent_stack)
    }
}

impl Diagnostic for ParserError {
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        match &self.kind {
            ParserErrorKind::LexerError(err) => err.code(),
            ParserErrorKind::UnexpectedToken { .. } => Some(Box::new("parser::unexpected_token")),
            ParserErrorKind::AmbiguousOperator { .. } => {
                Some(Box::new("parser::ambiguous_operator"))
            }
            ParserErrorKind::ExpectedToken { .. } => Some(Box::new("parser::expected_token")),
            ParserErrorKind::UnexpectedEof { .. } => Some(Box::new("parser::unexpected_eof")),
            ParserErrorKind::CustomError { code, .. } => {
                Some(Box::new(code.unwrap_or("parser::custom_error")))
            }
        }
    }

    fn severity(&self) -> Option<Severity> {
        Some(Severity::Error)
    }

    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        None
    }

    fn url<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        None
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = DiagnosticLabeledSpan> + '_>> {
        let labels = match &self.kind {
            ParserErrorKind::LexerError(err) => return err.labels(),
            ParserErrorKind::UnexpectedToken { span, .. } => vec![DiagnosticLabeledSpan::new(
                Some("Unexpected token".to_string()),
                span.start,
                span.len(),
            )],
            ParserErrorKind::AmbiguousOperator { span, .. } => vec![DiagnosticLabeledSpan::new(
                Some("Ambiguous operator".to_string()),
                span.start,
                span.len(),
            )],
            ParserErrorKind::ExpectedToken { span, expected, found } => {
                vec![DiagnosticLabeledSpan::new(
                    Some(format!("Expected {:?} here but got {:?}", expected, found)),
                    span.start,
                    span.len(),
                )]
            }
            ParserErrorKind::UnexpectedEof => return None,
            ParserErrorKind::CustomError { msg, span, .. } => vec![DiagnosticLabeledSpan::new(
                Some(msg.clone()),
                span.start,
                span.len(),
            )],
        };

        Some(Box::new(labels.into_iter()))
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn Diagnostic> + 'a>> {
        None
    }

    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        None
    }
}

pub type ParseResult<T = (NodeId, Span)> = osta_diagnostic::Result<T>;
pub type ParseResultOpt<T = (NodeId, Span)> = ParseResult<Option<T>>;

#[macro_export]
macro_rules! err {
    ($expr: expr) => {
        Err($crate::error::ParserError::from($expr).into())
    };
}
