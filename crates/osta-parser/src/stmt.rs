use crate::error::{eof_label, ParseResultOpt, ParserErrorPolicy};
use crate::expr::parse_expr;
use crate::item::parse_type;
use crate::util::{advance_if, expect, next, peek, unsafe_next};
use crate::{diagnostic, err, try_parse, FileSession, ParseResult};
use miette::{miette, LabeledSpan, Severity};
use osta_ast::ast::{Interned, InternedKind};
use osta_lexer::{Token, TokenKind};
use osta_syntax::Span;

pub fn parse_stmts() -> ParseResultOpt {
    let builder = FileSession::builder();

    if let Some(Token { kind: TokenKind::RBrace, .. }) = peek()? {
        return Ok(None);
    }

    // This parses a statement and recovers from errors
    // TODO(johan): emitting a warning of the skipped span could be beneficial
    let first = match parse_stmt() {
        Ok(stmt) => stmt,
        Err(err) => {
            loop {
                if let Ok(token) = next()
                    && matches!(token, Some(Token { kind: TokenKind::Semicolon, .. }) | None)
                {
                    break;
                }
                if let Ok(token) = peek()
                    && matches!(token, Some(Token { kind: TokenKind::RBrace, .. }))
                {
                    break;
                }
            }
            return Err(err);
        }
    };

    match try_parse!(parse_stmts) {
        Ok(Some((stmts_id, stmts_span))) => {
            let span = first.1.join(&stmts_span);
            let node_id = builder.add_chain(span.clone(), first.0, stmts_id);
            Ok(Some((node_id, span)))
        }
        _ => Ok(Some(first)),
    }
}

fn parse_stmt() -> ParseResult {
    let e1 = match try_parse!(parse_expr_stmt) {
        Err(err) => {
            if err.1 == ParserErrorPolicy::GoUp {
                return Err(err);
            } else {
                err
            }
        }
        ok => return ok,
    };

    let e2 = match try_parse!(parse_variable_binding) {
        Err(err) => {
            if err.1 == ParserErrorPolicy::GoUp {
                return Err(err);
            } else {
                err
            }
        }
        ok => return ok,
    };

    err!(
        diagnostic! {
            message = "Solve one of this problems",
            code = "parser/stmt",
            severity = Severity::Error,
            related = vec![e1.0, e2.0]
        },
        ParserErrorPolicy::Undefined
    )
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
            let Token { span, .. } = unsafe_next();
            err!(
                miette! {
                    severity = Severity::Error,
                    code = "parser/stmt/expr/termination",
                    labels = vec![LabeledSpan::new(
                        Some("`;` is missing after this expression".to_string()),
                        span.start,
                        span.end - span.start,
                    )],
                    "Unterminated expression"
                },
                ParserErrorPolicy::GoUp
            )
        }
        None => err!(
            miette! {
                severity = Severity::Error,
                code = "parser/stmt/expr/eof",
                labels = vec![eof_label("An expression was expected to start here")],
                "Unexpected end of file! An expression was expected"
            },
            ParserErrorPolicy::GoUp
        ),
    }
}

fn parse_variable_binding() -> ParseResult {
    let builder = FileSession::builder();

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
