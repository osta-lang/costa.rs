mod expr;
mod item;
mod path;
mod stmt;
#[cfg(test)]
mod tests;
mod util;

use crate::item::parse_item;
use crate::util::peek;
use osta_ast::{AstBuilder, NodeId, AST};
use osta_lexer::{Lexer, TokenKind};
use osta_session::Session;
use osta_syntax::Span;
use std::sync::{Arc, Mutex};
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
}

pub type ParseResult<T = (NodeId, Span)> = Result<T, ParserError>;
pub type ParseResultOpt<T = (NodeId, Span)> = Result<Option<T>, ParserError>;

pub fn parse(session: Arc<Mutex<Session>>, source: &str) -> Result<AST, ParserError> {
    let mut lexer = Lexer::new(source);
    let mut builder = AstBuilder::new();
    parse_root(session, &mut lexer, &mut builder)?;
    Ok(builder.build())
}

pub fn parse_root<'src>(
    session: Arc<Mutex<Session>>,
    lexer: &mut Lexer<'src>,
    builder: &mut AstBuilder,
) -> ParseResult<()> {
    while peek(lexer)?.is_some() {
        let id = builder
            .checkpoint()
            .resolve(parse_item(session.clone(), lexer, builder))?;
        builder.add_item(id);
    }
    Ok(())
}
