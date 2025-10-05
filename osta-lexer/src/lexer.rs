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
    #[error("unexpected EOF")]
    UnexpectedEof,
    #[error("unexpected token: expected {expected:?}, found {found:?}")]
    UnexpectedToken {
        expected: TokenKind,
        found: TokenKind,
    },
}

pub type LexResult<T = Token> = Result<T, LexerError>;

pub struct Lexer<'src> {
    stream: ::logos::SpannedIter<'src, TokenKind>,
    queue: Vec<LexResult>,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            stream: TokenKind::lexer(source).spanned(),
            queue: Vec::new(),
        }
    }

    pub fn source(&self) -> &'src str {
        self.stream.source()
    }

    pub fn peek(&mut self, n: usize) -> Option<&LexResult> {
        while self.queue.len() <= n {
            if let Some(result) = self.next() {
                self.queue.push(result)
            } else {
                return None;
            }
        }
        self.queue.get(n)
    }

    fn inner_next(&mut self) -> Option<LexResult> {
        if let Some(result) = self.queue.pop() {
            Some(result)
        } else if let Some((result, span)) = self.stream.next() {
            Some(result.map(|kind| Token::new(kind, span.into())))
        } else {
            None
        }
    }

    pub fn expect(&mut self, kind: TokenKind) -> LexResult {
        match self.peek(0) {
            Some(Ok(token)) if token.kind == kind => unsafe {
                self.inner_next().unwrap_unchecked()
            },
            Some(Ok(token)) => Err(LexerError::UnexpectedToken {
                expected: kind,
                found: token.kind.clone(),
            }),
            Some(Err(_)) => unsafe { self.inner_next().unwrap_unchecked() },
            None => Err(LexerError::UnexpectedEof),
        }
    }

    pub fn expect_opt(&mut self, kind: TokenKind) -> Option<LexResult> {
        match self.peek(0) {
            Some(Ok(token)) if token.kind == kind => self.inner_next(),
            Some(Ok(_)) => None,
            Some(Err(_)) => self.inner_next(),
            None => None,
        }
    }

    pub fn bump(&mut self) -> LexResult<()> {
        match self.inner_next() {
            Some(Ok(_)) => Ok(()),
            Some(Err(e)) => Err(e),
            None => Err(LexerError::UnexpectedEof),
        }
    }
}

impl<'src> Iterator for Lexer<'src> {
    type Item = LexResult;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner_next() {
                None => break None,
                Some(result) => match result {
                    Err(err) => break Some(Err(err)),
                    Ok(token) => {
                        if token.kind == TokenKind::Comment {
                            continue;
                        } else {
                            break Some(Ok(token));
                        }
                    }
                },
            }
        }
    }
}
