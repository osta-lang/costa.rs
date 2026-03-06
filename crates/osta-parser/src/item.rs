use crate::expr::{parse_block, parse_expr};
use crate::path::{continue_path, parse_ident};
use crate::util::{expect, next, peek};
use crate::{err, ParseResult, ParserError};
use osta_ast::ast::Ty;
use osta_ast::{AstBuilder, NodeId};
use osta_lexer::{Lexer, Token, TokenKind};
use osta_session::Session;
use osta_syntax::Span;

pub fn parse_item<'src>(
    session: &mut Session,
    lexer: &mut Lexer<'src>,
    builder: &mut AstBuilder,
) -> ParseResult<NodeId> {
    Ok(parse_fn_decl(session, lexer, builder)?.0)
}

pub fn parse_fn_decl<'src>(
    session: &mut Session,
    lexer: &mut Lexer<'src>,
    builder: &mut AstBuilder,
) -> ParseResult {
    let start = expect(lexer, TokenKind::Fn)?.span.start;
    let ident = parse_ident(session, lexer)?;
    let _ = expect(lexer, TokenKind::LParen)?;
    let _ = expect(lexer, TokenKind::RParen)?;
    let _ = expect(lexer, TokenKind::Arrow)?;
    let ty = parse_type(session, lexer, builder)?.0;
    let (body, block_span) = parse_block(session, lexer, builder)?;
    let end = block_span.end;

    let span = Span::new(start, end);
    let node = builder.add_fn_decl(span.clone(), ident, Some(ty), body);

    Ok((node, span))
}

pub fn parse_type<'src>(
    session: &mut Session,
    lexer: &mut Lexer<'src>,
    builder: &mut AstBuilder,
) -> ParseResult {
    let ty = match next(lexer)? {
        Some(Token { kind: TokenKind::Never, span }) => {
            (builder.add_type(span.clone(), Ty::Never), span)
        }
        Some(Token { kind: TokenKind::Void, span }) => {
            (builder.add_type(span.clone(), Ty::Void), span)
        }
        Some(Token { kind: TokenKind::UintType(size), span }) => {
            (builder.add_type(span.clone(), Ty::Uint(size)), span)
        }
        Some(Token { kind: TokenKind::IntType(size), span }) => {
            (builder.add_type(span.clone(), Ty::Int(size)), span)
        }
        Some(Token { kind: TokenKind::FloatType(size), span }) => {
            (builder.add_type(span.clone(), Ty::Float(size)), span)
        }
        Some(Token { kind: TokenKind::Identifier, span }) => {
            let (path_id, span) = continue_path(session, lexer, builder, span)?;
            (builder.add_type(span.clone(), Ty::Path(path_id)), span)
        }
        Some(Token { kind: TokenKind::Ampersand, span }) => {
            let (ty, span_next) = parse_type(session, lexer, builder)?;
            (
                builder.add_type(span.clone(), Ty::Reference(ty)),
                Span::new(span.start, span_next.end),
            )
        }
        Some(Token { kind: TokenKind::Star, span }) => {
            let (ty, span_next) = parse_type(session, lexer, builder)?;
            (
                builder.add_type(span.clone(), Ty::Pointer(ty)),
                Span::new(span.start, span_next.end),
            )
        }
        Some(Token { kind: TokenKind::LBracket, span }) => {
            let (ty, _) = parse_type(session, lexer, builder)?;

            let count = if let Some(Token { kind: TokenKind::Semicolon, .. }) = peek(lexer)? {
                next(lexer)?;
                let (number, _) = parse_expr(session, lexer, builder, 0)?;
                Some(number)
            } else {
                None
            };

            let terminator = if let Some(Token { kind: TokenKind::Colon, .. }) = peek(lexer)? {
                next(lexer)?;
                let (terminator, _) = parse_expr(session, lexer, builder, 0)?;
                Some(terminator)
            } else {
                None
            };

            let Token { span: rb_span, .. } = expect(lexer, TokenKind::RBracket)?;

            (
                builder.add_type(span.clone(), Ty::Array { ty, count, terminator }),
                Span::new(span.start, rb_span.end),
            )
        }
        Some(Token { kind, span }) => {
            return err!(ParserError::InvalidType { found: kind, span });
        }
        None => return err!(ParserError::UnexpectedEof),
    };

    Ok(ty)
}
