use crate::expr::parse_expr;
use crate::item::parse_type;
use crate::util::{expect, expect_opt, intern};
use crate::{try_parse, ParseResult, ParseResultOpt};
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
}

pub fn parse_stmts(
    session: Arc<Mutex<Session>>,
    lexer: &mut Lexer,
    builder: &mut AstBuilder,
    force: bool,
) -> ParseResultOpt {
    let first = match try_parse!(parse_stmt, session, lexer, builder) {
        Err(e) if force => return Err(e),
        Ok(first) => first,
        _ => return Ok(None),
    };
    match try_parse!(parse_stmts, session, lexer, builder, false) {
        Ok(Some(stmts)) => {
            let span = first.1.join(&stmts.1);
            let node_id = builder.add_chain(span.clone(), first.0, stmts.0);
            Ok(Some((node_id, span)))
        }
        _ => Ok(Some(first)),
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
    let opt_ty = match expect_opt(lexer, TokenKind::Colon)? {
        Some(_) => {
            lexer.next();
            Some(parse_type(session.clone(), lexer, builder)?.0)
        }
        None => None,
    };
    let opt_init = match expect_opt(lexer, TokenKind::Equal)? {
        Some(_) => {
            lexer.next();
            Some(parse_expr(session.clone(), lexer, builder, 0)?.0)
        }
        None => None,
    };
    let end = expect(lexer, TokenKind::Semicolon)?.span.end;

    let span = Span::new(start, end);
    let node_id = builder.add_variable_binding(span.clone(), ident, opt_ty, opt_init);
    Ok((node_id, span))
}
