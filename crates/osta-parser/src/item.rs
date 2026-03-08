use crate::expr::{parse_block, parse_expr};
use crate::path::{continue_path, create_path, parse_ident};
use crate::util::{advance_if, expect, next, peek};
use crate::{err, try_parse, ParseResult, ParserError};
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
    let args = try_parse!(parse_fn_decl_args, session, lexer, builder)
        .ok()
        .map(|(id, _)| id);
    let _ = expect(lexer, TokenKind::RParen)?;
    let ty = if advance_if(lexer, TokenKind::Arrow)? {
        let ty = parse_type(session, lexer, builder)?.0;
        Some(ty)
    } else {
        None
    };
    let (body, block_span) = parse_block(session, lexer, builder)?;
    let end = block_span.end;

    let span = Span::new(start, end);
    let node = builder.add_fn_decl(span.clone(), ident, args, ty, body);

    Ok((node, span))
}

pub fn parse_fn_decl_args(
    session: &mut Session,
    lexer: &mut Lexer,
    builder: &mut AstBuilder,
) -> ParseResult {
    let (this_id, this_span) = {
        let ident = parse_ident(session, lexer)?;
        expect(lexer, TokenKind::Colon)?;
        let ty = parse_type(session, lexer, builder)?;

        let span = ident.span.join(&ty.1);
        let id = builder.add_variable_binding(span.clone(), ident, Some(ty.0), None);
        (id, span)
    };
    match try_parse!(parse_fn_decl_args, session, lexer, builder) {
        Ok((next_id, next_span)) => {
            let span = this_span.join(&next_span);
            let node_id = builder.add_chain(span.clone(), this_id, next_id);
            Ok((node_id, span))
        }
        _ => Ok((this_id, this_span)),
    }
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
        Some(Token { kind: TokenKind::Type, span }) => {
            let path = create_path(
                session,
                builder,
                [
                    ("std", span.clone()),
                    ("comptime", span.clone()),
                    ("Type", span.clone()),
                ],
            );
            (builder.add_type(span.clone(), Ty::Path(path)), span)
        }
        Some(Token { kind, span }) => {
            return err!(ParserError::InvalidType { found: kind, span });
        }
        None => return err!(ParserError::UnexpectedEof),
    };

    Ok(ty)
}
