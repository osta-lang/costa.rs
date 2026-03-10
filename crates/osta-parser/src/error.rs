use crate::FileSession;
use miette::{Diagnostic, LabeledSpan, Report, Severity};
use osta_ast::NodeId;
use osta_syntax::Span;
use std::fmt::{Debug, Display};
use thiserror::Error;

#[derive(PartialEq, Debug)]
pub enum ParserErrorPolicy {
    Undefined,
    TryOthers,
    GoUp,
}

pub type ParserError = (Report, ParserErrorPolicy);
pub type ParseResult<T = (NodeId, Span)> = Result<T, ParserError>;
pub type ParseResultOpt<T = (NodeId, Span)> = ParseResult<Option<T>>;

#[derive(Debug, Error)]
#[error("[{severity:?} | {code}] {message}")]
pub(crate) struct VersatileError {
    message: String,
    code: &'static str,
    severity: Severity,
    pub(crate) help: Option<&'static str>,
    pub(crate) url: Option<&'static str>,
    pub(crate) labels: Vec<LabeledSpan>,
    pub(crate) related: Vec<Report>,
    pub(crate) cause: Option<Report>,
}

impl VersatileError {
    pub fn new(message: String, code: &'static str, severity: Severity) -> Self {
        Self {
            message,
            code,
            severity,
            help: None,
            url: None,
            labels: Vec::new(),
            related: Vec::new(),
            cause: None,
        }
    }
}

impl Diagnostic for VersatileError {
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        Some(Box::new(self.code))
    }

    fn severity(&self) -> Option<Severity> {
        Some(self.severity)
    }

    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.help.map(|s| Box::new(s) as Box<dyn Display + 'a>)
    }

    fn url<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.url.map(|s| Box::new(s) as Box<dyn Display + 'a>)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        if self.related.is_empty() {
            None
        } else {
            Some(Box::new(self.labels.iter().cloned()))
        }
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn Diagnostic> + 'a>> {
        use ::core::borrow::Borrow;
        Some(Box::new(
            self.related
                .iter()
                .map(|x| -> &(dyn Diagnostic) { x.borrow() }),
        ))
    }

    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        use ::core::borrow::Borrow;
        self.cause
            .as_ref()
            .map(|d| -> &(dyn Diagnostic) { d.borrow() })
    }
}

pub(crate) fn eof_label<S: Into<String>>(msg: S) -> LabeledSpan {
    LabeledSpan::new(Some(msg.into()), FileSession::lexer().source().len() - 1, 0)
}

#[macro_export]
macro_rules! err {
    ($diagnostic: expr, $policy: expr) => {{
        Err(($diagnostic.into(), $policy))
    }};
}

#[macro_export]
macro_rules! diagnostic {
    (
        message = $msg:expr,
        code = $code:literal,
        severity = $severity:expr
        $(, $field:ident = $value:expr)*
        $(,)?
    ) => {{
        let mut diag = $crate::error::VersatileError::new($msg.into(), $code, $severity);

        $($crate::diagnostic!(@set_field $field = $value, diag);)*

        diag
    }};
    (@set_field help = $value:literal, $diag:ident) => {
        $diag.help = Some($value);
    };
    (@set_field url = $value:literal, $diag:ident) => {
        $diag.url = Some($value);
    };
    (@set_field labels = $value:expr, $diag:ident) => {
        $diag.labels = $value;
    };
    (@set_field related = $value:expr, $diag:ident) => {
        $diag.related = $value;
    };
    (@set_field cause = $value:expr, $diag:ident) => {
        $diag.cause = Some($value);
    };
}
