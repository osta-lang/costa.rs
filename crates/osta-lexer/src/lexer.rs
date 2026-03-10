use crate::token::{Token, TokenKind};
use logos::Logos;
use osta_diagnostic::{Diagnostic, DiagnosticLabeledSpan};
use osta_syntax::Span;
use std::fmt::Display;
use thiserror::Error;

#[derive(Error, Default, Debug, PartialEq, Clone)]
pub enum LexerErrorKind {
    #[default]
    #[error("unknown token")]
    UnknownToken,
    #[error("invalid integer")]
    InvalidInteger(#[from] std::num::ParseIntError),
    #[error("unterminated block comment")]
    UnterminatedBlockComment,
    #[error("unterminated string literal")]
    UnterminatedString,
}

#[derive(Clone, Debug, Error)]
#[error("{kind}")]
pub struct LexerError {
    pub kind: LexerErrorKind,
    pub span: Span,
}

impl Diagnostic for LexerError {
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        let code = match self.kind {
            LexerErrorKind::UnknownToken => "lexer::unknown_token",
            LexerErrorKind::InvalidInteger(_) => "lexer::invalid_integer",
            LexerErrorKind::UnterminatedBlockComment => "lexer::unterminated_block_comment",
            LexerErrorKind::UnterminatedString => "lexer::unterminated_string",
        };

        Some(Box::new(code))
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = DiagnosticLabeledSpan> + '_>> {
        let labels = match self.kind {
            LexerErrorKind::UnknownToken => vec![DiagnosticLabeledSpan::new(
                Some("Unknown token".to_string()),
                self.span.start,
                self.span.len(),
            )],
            LexerErrorKind::InvalidInteger(_) => vec![DiagnosticLabeledSpan::new(
                Some("Invalid integer".to_string()),
                self.span.start,
                self.span.len(),
            )],
            LexerErrorKind::UnterminatedBlockComment => vec![DiagnosticLabeledSpan::new(
                Some("Unterminated block comment".to_string()),
                self.span.start,
                self.span.len(),
            )],
            LexerErrorKind::UnterminatedString => vec![DiagnosticLabeledSpan::new(
                Some("Unterminated string".to_string()),
                self.span.start,
                self.span.len(),
            )],
        };

        Some(Box::new(labels.into_iter()))
    }

    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        todo!()
    }
}

pub type LexResult<T = Token> = Result<T, LexerError>;

#[derive(Clone)]
pub struct Lexer<'src> {
    stream: ::logos::SpannedIter<'src, TokenKind>,
    accumulator: Vec<LexResult>,
    accumulator_idx: usize,
    checkpoints: Vec<usize>,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            stream: TokenKind::lexer(source).spanned(),
            accumulator: Vec::new(),
            accumulator_idx: 0,
            checkpoints: Vec::new(),
        }
    }

    pub fn source(&self) -> &'src str {
        self.stream.source()
    }

    pub fn checkpoint(&mut self) {
        self.checkpoints.push(self.accumulator_idx);
    }

    pub fn commit(&mut self) {
        self.checkpoints.pop();
        self.collapse();
    }

    pub fn rollback(&mut self) {
        if let Some(idx) = self.checkpoints.pop() {
            self.accumulator_idx = idx;
        }
    }

    pub fn peek(&mut self) -> Option<&LexResult> {
        if self.accumulator.get(self.accumulator_idx).is_none() {
            let next = self.advance()?;
            self.accumulator.push(next);
        }
        self.accumulator.get(self.accumulator_idx)
    }

    fn advance(&mut self) -> Option<LexResult> {
        loop {
            match self.stream.next() {
                None => break None,
                Some((result, span)) => match result {
                    Err(err) => break Some(Err(LexerError { kind: err, span: span.into() })),
                    Ok(kind) => {
                        if kind == TokenKind::Comment {
                            continue;
                        } else {
                            break Some(Ok(Token::new(kind, span.into())));
                        }
                    }
                },
            }
        }
    }

    fn collapse(&mut self) {
        if self.checkpoints.is_empty() && self.accumulator_idx == self.accumulator.len() {
            self.accumulator.clear();
            self.accumulator_idx = 0;
        }
    }
}

impl<'src> Iterator for Lexer<'src> {
    type Item = LexResult;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(entry) = self.accumulator.get(self.accumulator_idx) {
            self.accumulator_idx += 1;

            if self.checkpoints.is_empty() && self.accumulator_idx == self.accumulator.len() {
                let entry = self.accumulator.pop();
                self.accumulator.clear();
                self.accumulator_idx = 0;
                return entry;
            }

            return Some(entry.clone());
        }

        let entry = self.advance()?;

        if self.checkpoints.len() > 0 {
            self.accumulator.push(entry.clone());
            self.accumulator_idx += 1;
        }

        Some(entry)
    }
}
