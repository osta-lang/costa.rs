use crate::expr::parse_expr;
use crate::item::parse_type;
use crate::util::{advance_if, expect, intern, next_if};
use crate::{try_parse, ParseResult};
use osta_ast::ast::{Interned, InternedKind};
use osta_ast::AstBuilder;
use osta_lexer::{Lexer, TokenKind};
use osta_session::Session;
use osta_syntax::Span;
use std::sync::{Arc, Mutex};

pub fn parse_stmt(
    session: Arc<Mutex<Session>>,
    lexer: &mut Lexer,
    builder: &mut AstBuilder,
) -> ParseResult {
    try_parse!(parse_variable_binding, session, lexer, builder, true)
        .or_else(|_| try_parse!(parse_expr_stmt, session, lexer, builder))
}

pub fn parse_stmts(
    session: Arc<Mutex<Session>>,
    lexer: &mut Lexer,
    builder: &mut AstBuilder,
) -> ParseResult {
    let first = parse_stmt(session.clone(), lexer, builder)?;
    match try_parse!(parse_stmts, session, lexer, builder) {
        Ok((stmts_id, stmts_span)) => {
            let span = first.1.join(&stmts_span);
            let node_id = builder.add_chain(span.clone(), first.0, stmts_id);
            Ok((node_id, span))
        }
        _ => Ok(first),
    }
}

pub fn parse_variable_binding(
    session: Arc<Mutex<Session>>,
    lexer: &mut Lexer,
    builder: &mut AstBuilder,
    with_let: bool,
) -> ParseResult {
    let (start, ident) = {
        let (start, span) = if with_let {
            let start = expect(lexer, TokenKind::Let)?.span.start;
            let span = expect(lexer, TokenKind::Identifier)?.span;
            (start, span)
        } else {
            let span = expect(lexer, TokenKind::Identifier)?.span;
            (span.start, span)
        };
        let idx = intern(session.clone(), lexer, &span);
        (start, Interned::new(idx, span, InternedKind::Ident))
    };
    let opt_ty = if advance_if(lexer, TokenKind::Colon)? {
        Some(parse_type(session.clone(), lexer, builder)?.0)
    } else {
        None
    };
    let opt_init = if advance_if(lexer, TokenKind::Equal)? {
        Some(parse_expr(session.clone(), lexer, builder, 0)?.0)
    } else {
        None
    };
    let end = expect(lexer, TokenKind::Semicolon)?.span.end;

    let span = Span::new(start, end);
    let node_id = builder.add_variable_binding(span.clone(), ident, opt_ty, opt_init);
    Ok((node_id, span))
}

pub fn parse_expr_stmt(
    session: Arc<Mutex<Session>>,
    lexer: &mut Lexer,
    builder: &mut AstBuilder,
) -> ParseResult {
    let (expr_id, expr_span) = parse_expr(session, lexer, builder, 0)?;
    let span = next_if(lexer, TokenKind::Semicolon)?
        .map(|token| expr_span.join(&token.span))
        .unwrap_or(expr_span);
    Ok((expr_id, span))
}
