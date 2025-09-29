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

pub type TokenResult<'src> = Result<Token, LexerError>;

pub struct Lexer<'src> {
    stream: ::logos::SpannedIter<'src, TokenKind>,
    queue: Vec<TokenResult<'src>>,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            stream: TokenKind::lexer(source).spanned(),
            queue: Vec::new(),
        }
    }

    pub fn peek(&mut self, n: usize) -> Option<&TokenResult<'src>> {
        while self.queue.len() <= n {
            if let Some(result) = self.next() {
                self.queue.push(result)
            } else {
                return None;
            }
        }
        self.queue.get(n)
    }

    fn inner_next(&mut self) -> Option<TokenResult<'src>> {
        if let Some(result) = self.queue.pop() {
            Some(result)
        } else if let Some((result, span)) = self.stream.next() {
            Some(result.map(|kind| Token::new(kind, span.into())))
        } else {
            None
        }
    }
}

impl<'src> Iterator for Lexer<'src> {
    type Item = TokenResult<'src>;

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
                }
            }
        }
    }
}
