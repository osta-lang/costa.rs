use crate::error::{eof_label, ParseResultOpt, ParserErrorPolicy};
use crate::path::{continue_path, parse_path};
use crate::stmt::parse_stmts;
use crate::util::{advance_if, expect, expect_opt, next, peek};
use crate::{err, try_parse, FileSession, ParseResult};
use miette::miette;
use miette::{LabeledSpan, Severity};
use osta_ast::ast::{Interned, InternedKind};
use osta_lexer::{Token, TokenKind};
use osta_syntax::Span;

pub fn parse_expr<'src>(min_bp: u8) -> ParseResult {
    let builder = FileSession::builder();

    macro_rules! postfix_op {
        ($builder:expr, $lhs:ident, $l_bp:literal, $($path:ident)::+) => {{
            let Token { span, .. } = unsafe { next()?.unwrap_unchecked() };
            if $l_bp < min_bp {
                break;
            }
            let path = $crate::path::create_path([
                $((stringify!($path), $lhs.1.clone())),+
            ]);
            let span = $lhs.1.join(&span);
            let node = $builder.add_fn_call(span.clone(), path, Some($lhs.0));
            (node, span)
        }};
    }

    macro_rules! infix_op {
        ($builder:expr, $lhs:ident, $l_bp:literal, $r_bp:literal, $($path:ident)::+) => {{
            let checkpoint = $crate::util::Checkpoint::new();
            let Token { span, .. } = $crate::util::unsafe_next();
            if $l_bp < min_bp {
                checkpoint.rollback();
                break;
            } else if $l_bp == min_bp {
                checkpoint.rollback();
                return err!(
                    miette! {
                        severity = Severity::Error,
                        code = "parser/expr/ambiguous_op",
                        labels = vec![LabeledSpan::new(
                            Some("This operator is ambiguous because of its precedence".to_string()),
                            span.start,
                            span.end - span.start,
                        )],
                        help = "Adding parenthesis may help",
                        "Ambiguous operator"
                    },
                    ParserErrorPolicy::Undefined
                );
            }
            let (rhs, rhs_span) = checkpoint.resolve(parse_expr($r_bp))?;
            let path = $crate::path::create_path([
                $((stringify!($path), span.clone())),+
            ]);
            let span = $lhs.1.join(&rhs_span);
            let args = $builder.add_chain(span.clone(), $lhs.0, rhs);
            let node = $builder.add_fn_call(span.clone(), path, Some(args));
            (node, span)
        }};
    }

    let mut lhs = match peek()? {
        Some(Token { kind: TokenKind::LBrace, .. }) => parse_block()?,
        Some(_) => parse_expr_lhs()?,
        None => {
            return err!(
                miette! {
                    severity = Severity::Error,
                    code = "parser/expr/eof",
                    labels = vec![eof_label("An expression was expected to begin here, but instead the end of file was reached")],
                    "Unexpected end of file! Expression was expected"
                },
                ParserErrorPolicy::Undefined
            )
        }
    };

    loop {
        lhs = match peek()? {
            Some(Token { kind: TokenKind::LParen, .. }) => {
                next()?;
                let args = parse_fn_call_args()?.map(|(id, _)| id);
                let span = lhs.1.join(&expect(TokenKind::RParen)?.span);
                (builder.add_fn_call(span.clone(), lhs.0, args), span)
            }
            Some(Token { kind: TokenKind::DoublePlus, .. }) => {
                postfix_op!(builder, lhs, 11, core::ops::PostInc::inc)
            }
            Some(Token { kind: TokenKind::DoubleMinus, .. }) => {
                postfix_op!(builder, lhs, 11, core::ops::PostDec::dec)
            }
            Some(Token { kind: TokenKind::Question, .. }) => {
                let Token { span: lhs_span, .. } = unsafe { next()?.unwrap_unchecked() };
                if 11 < min_bp {
                    break;
                }

                if let Ok((expr1, expr1_span)) = try_parse!(parse_expr, 0) {
                    match peek()? {
                        Some(Token { kind: TokenKind::Colon, .. }) => {
                            let _ = unsafe { next()?.unwrap_unchecked() };

                            let (expr2, expr2_span) = parse_expr(0)?;

                            let span = lhs.1.join(&expr2_span);
                            let node = builder.add_if_expr(span.clone(), lhs.0, expr1, Some(expr2));
                            (node, span)
                        }
                        _ => {
                            let some_path = crate::path::create_path([
                                ("core", lhs.1.clone()),
                                ("option", lhs.1.clone()),
                                ("Option", lhs.1.clone()),
                                ("Some", lhs.1.clone()),
                            ]);
                            let none_path = crate::path::create_path([
                                ("core", lhs.1.clone()),
                                ("option", lhs.1.clone()),
                                ("Option", lhs.1.clone()),
                                ("None", lhs.1.clone()),
                            ]);
                            let span = lhs.1.join(&expr1_span);
                            let some_node =
                                builder.add_variant_inst(span.clone(), some_path, Some(expr1));
                            let none_node = builder.add_variant_inst(span.clone(), none_path, None);
                            let node = builder.add_if_expr(
                                span.clone(),
                                lhs.0,
                                some_node,
                                Some(none_node),
                            );
                            (node, span)
                        }
                    }
                } else {
                    let path = crate::path::create_path([
                        ("core", lhs.1.clone()),
                        ("ops", lhs.1.clone()),
                        ("Try", lhs.1.clone()),
                        ("unwrap", lhs.1.clone()),
                    ]);
                    let span = lhs.1.join(&lhs_span);
                    let node = builder.add_fn_call(span.clone(), path, Some(lhs.0));
                    (node, span)
                }
            }
            Some(Token { kind: TokenKind::Plus, .. }) => {
                infix_op!(builder, lhs, 5, 6, core::ops::Addition::add)
            }
            Some(Token { kind: TokenKind::Minus, .. }) => {
                infix_op!(builder, lhs, 5, 6, core::ops::Subtraction::sub)
            }
            Some(Token { kind: TokenKind::Star, .. }) => {
                infix_op!(builder, lhs, 7, 8, core::ops::Multiplication::mul)
            }
            Some(Token { kind: TokenKind::Slash, .. }) => {
                infix_op!(builder, lhs, 7, 8, core::ops::Division::div)
            }
            Some(Token { kind: TokenKind::Percent, .. }) => {
                infix_op!(builder, lhs, 7, 8, core::ops::Reminder::rem)
            }
            Some(Token { kind: TokenKind::Ampersand, .. }) => {
                infix_op!(builder, lhs, 3, 4, core::ops::BitAnd::and)
            }
            Some(Token { kind: TokenKind::Pipe, .. }) => {
                infix_op!(builder, lhs, 1, 2, core::ops::BitOr::or)
            }
            Some(Token { kind: TokenKind::Caret, .. }) => {
                infix_op!(builder, lhs, 2, 3, core::ops::BitXor::xor)
            }
            Some(Token { kind: TokenKind::LShift, .. }) => {
                infix_op!(builder, lhs, 6, 7, core::ops::Shl::shl)
            }
            Some(Token { kind: TokenKind::RShift, .. }) => {
                infix_op!(builder, lhs, 6, 7, core::ops::Shr::shr)
            }
            Some(Token { kind: TokenKind::ARShift, .. }) => {
                infix_op!(builder, lhs, 6, 7, core::ops::Shr::arshr)
            }
            Some(Token { kind: TokenKind::DoubleAmpersand, .. }) => {
                infix_op!(builder, lhs, 9, 10, core::ops::LogicalAnd::and)
            }
            Some(Token { kind: TokenKind::DoublePipe, .. }) => {
                infix_op!(builder, lhs, 9, 10, core::ops::LogicalOr::or)
            }
            Some(Token { kind: TokenKind::DoubleEqual, .. }) => {
                infix_op!(builder, lhs, 4, 5, core::ops::Eq::eq)
            }
            Some(Token { kind: TokenKind::NotEqual, .. }) => {
                infix_op!(builder, lhs, 4, 5, core::ops::Eq::ne)
            }
            Some(Token { kind: TokenKind::Less, .. }) => {
                infix_op!(builder, lhs, 4, 5, core::ops::Ord::lt)
            }
            Some(Token { kind: TokenKind::LessEqual, .. }) => {
                infix_op!(builder, lhs, 4, 5, core::ops::Ord::le)
            }
            Some(Token { kind: TokenKind::Greater, .. }) => {
                infix_op!(builder, lhs, 4, 5, core::ops::Ord::gt)
            }
            Some(Token { kind: TokenKind::GreaterEqual, .. }) => {
                infix_op!(builder, lhs, 4, 5, core::ops::Ord::ge)
            }
            Some(Token { kind: TokenKind::DoubleDot, .. }) => {
                infix_op!(builder, lhs, 11, 11, core::ops::Range::new)
            }
            None | Some(_) => break,
        };
    }

    Ok(lhs)
}

fn parse_expr_lhs<'src>() -> ParseResult {
    let builder = FileSession::builder();

    macro_rules! prefix_op {
        ($builder:expr, $op_span:ident, $($path:ident)::+) => {{
            let (rhs, rhs_span) = parse_expr(9)?;
            let path = $crate::path::create_path([
                $((stringify!($path), $op_span.clone())),+
            ]);
            let span = $op_span.join(&rhs_span);
            let node = $builder.add_fn_call(span.clone(), path, Some(rhs));
            (node, span)
        }};
    }

    let lhs = match next()? {
        Some(Token { kind: TokenKind::LParen, span: lparen_span }) => {
            let lhs = parse_expr(0)?.0;
            let rparen_span = expect(TokenKind::RParen)?.span;
            let span = lparen_span.join(&rparen_span);
            (lhs, span)
        }
        Some(Token { kind: TokenKind::If, span: if_span }) => {
            let cond_id = parse_expr(0)?.0;
            let (then_id, then_span) = parse_expr(0)?;
            let (else_opt, span) = if advance_if(TokenKind::Else)? {
                let (else_id, else_span) = try_parse!(parse_expr, 0)?;
                let span = if_span.join(&else_span);
                (Some(else_id), span)
            } else {
                (None, if_span.join(&then_span))
            };
            let node_id = builder.add_if_expr(span.clone(), cond_id, then_id, else_opt);
            (node_id, span)
        }
        Some(Token { kind, span }) => match kind {
            TokenKind::DecInt => {
                let idx = FileSession::intern(&span);
                let node = builder.add_literal(span.clone(), idx, InternedKind::DecInt);
                (node, span)
            }
            TokenKind::BinInt => {
                let idx = FileSession::intern(&span);
                let node = builder.add_literal(span.clone(), idx, InternedKind::BinInt);
                (node, span)
            }
            TokenKind::OctInt => {
                let idx = FileSession::intern(&span);
                let node = builder.add_literal(span.clone(), idx, InternedKind::OctInt);
                (node, span)
            }
            TokenKind::HexInt => {
                let idx = FileSession::intern(&span);
                let node = builder.add_literal(span.clone(), idx, InternedKind::HexInt);
                (node, span)
            }
            TokenKind::Float => {
                let idx = FileSession::intern(&span);
                let node = builder.add_literal(span.clone(), idx, InternedKind::Float);
                (node, span)
            }
            TokenKind::IntFloat => {
                let idx = FileSession::intern(&span);
                let node = builder.add_literal(span.clone(), idx, InternedKind::IntFloat);
                (node, span)
            }
            TokenKind::FloatExp => {
                let idx = FileSession::intern(&span);
                let node = builder.add_literal(span.clone(), idx, InternedKind::FloatExp);
                (node, span)
            }
            TokenKind::IntExp => {
                let idx = FileSession::intern(&span);
                let node = builder.add_literal(span.clone(), idx, InternedKind::IntExp);
                (node, span)
            }
            TokenKind::String => {
                let idx = FileSession::intern(&span);
                let node = builder.add_literal(span.clone(), idx, InternedKind::Str);
                (node, span)
            }
            TokenKind::RawString => {
                let idx = FileSession::intern(&span);
                let node = builder.add_literal(span.clone(), idx, InternedKind::RawStr);
                (node, span)
            }
            TokenKind::Char => {
                let idx = FileSession::intern(&span);
                let node = builder.add_literal(span.clone(), idx, InternedKind::Char);
                (node, span)
            }
            TokenKind::Identifier => continue_path(span)?,
            TokenKind::MacroIdentifier | TokenKind::ComptimeIdentifier => {
                let idx = FileSession::intern(&span);
                let interned = Interned::new(idx, span.clone(), InternedKind::Ident);
                let node = builder.add_path(span.clone(), interned, None);
                (node, span)
            }
            TokenKind::DoubleColon => {
                let (idx, path_span) = parse_path(false)?;
                (idx, span.join(&path_span))
            }
            TokenKind::Minus => {
                prefix_op!(builder, span, core::ops::Neg::neg)
            }
            TokenKind::Star => {
                prefix_op!(builder, span, core::ops::Deref::deref)
            }
            TokenKind::Bang => {
                prefix_op!(builder, span, core::ops::LogicalNot::not)
            }
            TokenKind::Tilde => {
                prefix_op!(builder, span, core::ops::BitNot::not)
            }
            TokenKind::DoublePlus => {
                prefix_op!(builder, span, core::ops::PreInc::inc)
            }
            TokenKind::DoubleMinus => {
                prefix_op!(builder, span, core::ops::PreDec::dec)
            }
            kind => {
                return err!(
                    miette! {
                        severity = Severity::Error,
                        code = "parser/expr/unknown_starter",
                        labels = vec![LabeledSpan::new(
                            Some("Change this by a prefix operator or a value".into()),
                            span.start,
                            span.end - span.start,
                        )],
                        "Expression can't start with `{:?}`", kind
                    },
                    ParserErrorPolicy::Undefined
                )
            }
        },
        None => {
            return err!(
                miette! {
                    severity = Severity::Error,
                    code = "parser/expr/eof_starter",
                    labels = vec![eof_label("An expression was expected here")],
                    "Unexpected end of file! Expression was expected"
                },
                ParserErrorPolicy::Undefined
            )
        }
    };

    Ok(lhs)
}

pub fn parse_block<'src>() -> ParseResult {
    let builder = FileSession::builder();

    let start = expect(TokenKind::LBrace)?.span.start;
    // let stmts = try_parse!(parse_stmts, session, lexer, builder)
    //     .ok()
    //     .map(|(node_id, _)| node_id);
    // TODO(johan): this is the correct implementation ^ Comment this v
    let stmts = Some(parse_stmts()?.0);
    let end = expect(TokenKind::RBrace)?.span.end;

    let span = Span::new(start, end);
    let node = builder.add_block(span.clone(), stmts);
    Ok((node, span))
}

fn parse_fn_call_args() -> ParseResultOpt {
    let builder = FileSession::builder();

    if expect_opt(TokenKind::RParen)?.is_some() {
        return Ok(None);
    }

    let first = parse_expr(0)?;
    let out = if advance_if(TokenKind::Comma)?
        && let Some(next) = parse_fn_call_args()?
    {
        let span = first.1.join(&next.1);
        (builder.add_chain(span.clone(), first.0, next.0), span)
    } else {
        first
    };
    Ok(Some(out))
}
