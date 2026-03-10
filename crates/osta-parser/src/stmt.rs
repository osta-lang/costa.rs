use crate::error::ParserIntent;
use crate::expr::parse_expr;
use crate::item::parse_type;
use crate::util::{advance_if, expect, next, peek, unsafe_next};
use crate::{err, scoped_intent, try_parse, FileSession, ParseResult, ParserErrorKind};
use osta_ast::ast::{Interned, InternedKind};
use osta_diagnostic::RequireSolvingOne;
use osta_lexer::{Token, TokenKind};
use osta_syntax::Span;

pub fn parse_stmts() -> ParseResult {
    let builder = FileSession::ast();

    let first = match parse_stmt() {
        Ok(stmt) => stmt,
        err => {
            loop {
                if let Err(_) = peek() {
                    let _ = next();
                    continue;
                }
                let Token { kind, .. } = unsafe_next();
                if matches!(kind, TokenKind::Semicolon) {
                    break;
                }
            }
            return err;
        }
    };

    match try_parse!(parse_stmts) {
        Ok((stmts_id, stmts_span)) => {
            let span = first.1.join(&stmts_span);
            let node_id = builder.add_chain(span.clone(), first.0, stmts_id);
            Ok((node_id, span))
        }
        _ => Ok(first),
    }
}

fn parse_stmt() -> ParseResult {
    scoped_intent!(ParserIntent::Statement);

    let e1 = match try_parse!(parse_expr_stmt) {
        Err(e1) => e1,
        ok => return ok,
    };

    let e2 = match try_parse!(parse_variable_binding) {
        Err(e2) => e2,
        ok => return ok,
    };

    Err(RequireSolvingOne::new(vec![e1, e2]).into())
}

fn parse_expr_stmt() -> ParseResult {
    let (expr_id, expr_span) = parse_expr(0)?;

    match peek()? {
        Some(Token { kind: TokenKind::Semicolon, span }) => {
            let span = expr_span.join(span);
            unsafe_next();
            Ok((expr_id, span))
        }
        Some(Token { kind: TokenKind::RBrace, .. }) => Ok((expr_id, expr_span)),
        Some(_) => {
            let Token { kind, span } = unsafe_next();
            err!(ParserErrorKind::ExpectedToken {
                expected: TokenKind::Semicolon,
                found: kind,
                span
            })
        }
        None => err!(ParserErrorKind::UnexpectedEof),
    }
}

fn parse_variable_binding() -> ParseResult {
    let builder = FileSession::ast();

    let (start, ident) = {
        let start = expect(TokenKind::Let)?.span.start;
        let span = expect(TokenKind::Identifier)?.span;
        let idx = FileSession::intern(&span);
        (start, Interned::new(idx, span, InternedKind::Ident))
    };
    let opt_ty = if advance_if(TokenKind::Colon)? {
        Some(parse_type()?.0)
    } else {
        None
    };
    let opt_init = if advance_if(TokenKind::Equal)? {
        Some(parse_expr(0)?.0)
    } else {
        None
    };
    let end = expect(TokenKind::Semicolon)?.span.end;

    let span = Span::new(start, end);
    let node_id = builder.add_variable_binding(span.clone(), ident, opt_ty, opt_init);
    Ok((node_id, span))
}
