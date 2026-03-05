use crate::token::{Token, TokenKind};
use logos::Logos;
use thiserror::Error;

#[derive(Error, Default, Debug, PartialEq, Clone)]
pub enum LexerError {
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

pub type LexResult<T = Token> = Result<T, LexerError>;

#[derive(Clone)]
pub struct Lexer<'src> {
    stream: ::logos::SpannedIter<'src, TokenKind>,
    peeked: Option<LexResult>,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str) -> Self {
        Self { stream: TokenKind::lexer(source).spanned(), peeked: None }
    }

    pub fn source(&self) -> &'src str {
        self.stream.source()
    }

    pub fn peek(&mut self) -> Option<&LexResult> {
        if self.peeked.is_none() {
            self.peeked = self.next();
        }
        self.peeked.as_ref()
    }
}

impl<'src> Iterator for Lexer<'src> {
    type Item = LexResult;

    fn next(&mut self) -> Option<Self::Item> {
        if self.peeked.is_some() {
            return self.peeked.take();
        }

        self.peeked = None;

        loop {
            match self.stream.next() {
                None => break None,
                Some((result, span)) => match result {
                    Err(err) => break Some(Err(err)),
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
}
