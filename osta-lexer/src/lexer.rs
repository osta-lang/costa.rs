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
}

pub type TokenResult<'src> = Result<Token<'src>, LexerError>;

pub struct Lexer<'src> {
    stream: ::logos::Lexer<'src, TokenKind>,
    queue: Vec<TokenResult<'src>>,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            stream: TokenKind::lexer(source),
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

    #[cfg(test)]
    pub(crate) fn slice(&self) -> &'src str {
        self.stream.slice()
    }

    fn inner_next(&mut self) -> Option<TokenResult<'src>> {
        if let Some(result) = self.queue.pop() {
            Some(result)
        } else if let Some(result) = self.stream.next() {
            Some(result.map(|kind| Token::new(kind, self.stream.slice())))
        } else {
            None
        }
    }
}

impl<'src> Iterator for Lexer<'src> {
    type Item = TokenResult<'src>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner_next()
        // TODO(johan): add macro expansion here
        /*
        TODO(johan): implement operators here
        Operators in osta are not necessarily hardcoded tokens, they can be sequences of other
        tokens. For example, the operator `->` is hardcoded, but `+` is not, it must be emitted
        redirecting the unrecognized token as an operator token. The token `++` can collide with
        `+`, and this patterns can be arbitrarily long and unknown by us, so we must consider a
        Trie structure or an ART to store operators and match them greedily.
        */
    }
}
