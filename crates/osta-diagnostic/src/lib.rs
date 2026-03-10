use ::miette::*;
pub use ::miette::{
    Diagnostic, LabeledSpan as DiagnosticLabeledSpan, Result, Severity,
    SourceCode as DiagnosticSourceCode, SourceSpan as DiagnosticSpan,
};
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
#[error("all errors must be resolved")]
#[diagnostic(code(diagnostic::all))]
pub struct RequireSolvingAll {
    #[related]
    issues: Vec<Report>,
}

impl RequireSolvingAll {
    pub fn new(issues: Vec<Report>) -> Self {
        Self { issues }
    }
}

#[derive(Error, Debug, Diagnostic)]
#[error("one of this errors must be resolved")]
#[diagnostic(code(diagnostic::one))]
pub struct RequireSolvingOne {
    #[related]
    issues: Vec<Report>,
}

impl RequireSolvingOne {
    pub fn new(issues: Vec<Report>) -> Self {
        Self { issues }
    }
}
