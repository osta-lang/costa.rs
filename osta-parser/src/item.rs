use crate::expr::parse_block;
use crate::path::{continue_path, parse_ident};
use crate::util::{expect, next};
use crate::{ParseResult, ParserError};
use osta_ast::ast::{Ty, TyKind};
use osta_ast::{AstBuilder, NodeId};
use osta_lexer::{Lexer, Token, TokenKind};
use osta_session::Session;
use osta_syntax::Span;
use std::sync::{Arc, Mutex};

pub fn parse_item<'src>(
    session: Arc<Mutex<Session>>,
    lexer: &mut Lexer<'src>,
    builder: &mut AstBuilder,
) -> ParseResult<NodeId> {
    Ok(parse_fn_decl(session, lexer, builder)?.0)
}

pub fn parse_fn_decl<'src>(
    session: Arc<Mutex<Session>>,
    lexer: &mut Lexer<'src>,
    builder: &mut AstBuilder,
) -> ParseResult {
    let start = expect(lexer, TokenKind::Fn)?.span.start;
    let ident = parse_ident(session.clone(), lexer)?;
    let _ = expect(lexer, TokenKind::LParen)?;
    let _ = expect(lexer, TokenKind::RParen)?;
    let _ = expect(lexer, TokenKind::Arrow)?;
    let ty = parse_type(session.clone(), lexer, builder)?.0;
    let (body, block_span) = parse_block(session, lexer, builder)?;
    let end = block_span.end;

    let span = Span::new(start, end);
    let node = builder.add_fn_decl(span.clone(), ident, Some(ty), body);

    Ok((node, span))
}

pub fn parse_type<'src>(
    session: Arc<Mutex<Session>>,
    lexer: &mut Lexer<'src>,
    builder: &mut AstBuilder,
) -> ParseResult<(Ty, Span)> {
    let (ty, span) = match next(lexer)? {
        Some(Token { kind: TokenKind::Never, span }) => (TyKind::Never, span),
        Some(Token { kind: TokenKind::Void, span }) => (TyKind::Void, span),
        Some(Token { kind: TokenKind::UintType(size), span }) => (TyKind::Uint(size), span),
        Some(Token { kind: TokenKind::IntType(size), span }) => (TyKind::Int(size), span),
        Some(Token { kind: TokenKind::FloatType(size), span }) => (TyKind::Float(size), span),
        Some(Token { kind: TokenKind::Identifier, span }) => {
            let (path_id, span) = continue_path(session, lexer, builder, span)?;
            (TyKind::Other(path_id), span)
        }
        Some(Token { kind, span }) => {
            return Err(ParserError::InvalidType { found: kind, span });
        }
        None => return Err(ParserError::UnexpectedEof),
    };
    let ty = Ty { span: span.clone(), kind: ty };
    Ok((ty, span))
}
