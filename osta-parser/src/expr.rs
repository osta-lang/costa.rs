use crate::path::continue_path;
use crate::util::{expect, intern, next, peek};
use crate::{ParseResult, ParserError};
use osta_ast::ast::InternedKind;
use osta_ast::AstBuilder;
use osta_lexer::{Lexer, Token, TokenKind};
use osta_session::Session;
use osta_syntax::Span;
use std::sync::{Arc, Mutex};

pub fn parse_expr<'src>(
    session: Arc<Mutex<Session>>,
    lexer: &mut Lexer<'src>,
    builder: &mut AstBuilder,
    min_bp: u8,
) -> ParseResult {
    macro_rules! prefix_op {
        ($session:expr, $lexer:expr, $builder:expr, $op_span:ident, $($path:ident)::+) => {{
            let (rhs, rhs_span) = parse_expr($session, $lexer, $builder, 9)?;
            let path = $crate::path::create_path($session, $builder, [
                $((stringify!($path), $op_span.clone())),+
            ]);
            let span = $op_span.join(&rhs_span);
            let node = $builder.add_fn_call(span.clone(), path, rhs);
            (node, span)
        }};
    }

    macro_rules! postfix_op {
        ($session:expr, $lexer:expr, $builder:expr, $lhs:ident, $l_bp:literal, $($path:ident)::+) => {{
            let Token { span, .. } = unsafe { next($lexer)?.unwrap_unchecked() };
            if $l_bp < min_bp {
                break;
            }
            let path = $crate::path::create_path($session, $builder, [
                $((stringify!($path), $lhs.1.clone())),+
            ]);
            let span = $lhs.1.join(&span);
            let node = $builder.add_fn_call(span.clone(), path, $lhs.0);
            (node, span)
        }};
    }

    macro_rules! infix_op {
        ($session:expr, $lexer:expr, $builder:expr, $lhs:ident, $l_bp:literal, $r_bp:literal, $($path:ident)::+) => {{
            let Token { span, .. } = unsafe { next($lexer)?.unwrap_unchecked() };
            if $l_bp < min_bp {
                break;
            } else if $l_bp == min_bp {
                return Err(ParserError::AmbiguousOperator { span });
            }
            let (rhs, rhs_span) = parse_expr($session, $lexer, $builder, $r_bp)?;
            let path = $crate::path::create_path($session, $builder, [
                $((stringify!($path), span.clone())),+
            ]);
            let span = $lhs.1.join(&rhs_span);
            let args = $builder.add_chain(span.clone(), $lhs.0, rhs);
            let node = $builder.add_fn_call(span.clone(), path, args);
            (node, span)
        }};
    }

    let mut lhs = match next(lexer)? {
        Some(Token { kind: TokenKind::LParen, span: lparen_span }) => {
            let lhs = parse_expr(session.clone(), lexer, builder, 0)?.0;
            let rparen_span = expect(lexer, TokenKind::RParen)?.span;
            let span = lparen_span.join(&rparen_span);
            (lhs, span)
        }
        Some(Token { kind, span }) => match kind {
            TokenKind::DecInt => {
                let idx = intern(session.clone(), lexer, &span);
                let node = builder.add_literal(span.clone(), idx, InternedKind::DecInt);
                (node, span)
            }
            TokenKind::BinInt => {
                let idx = intern(session.clone(), lexer, &span);
                let node = builder.add_literal(span.clone(), idx, InternedKind::BinInt);
                (node, span)
            }
            TokenKind::OctInt => {
                let idx = intern(session.clone(), lexer, &span);
                let node = builder.add_literal(span.clone(), idx, InternedKind::OctInt);
                (node, span)
            }
            TokenKind::HexInt => {
                let idx = intern(session.clone(), lexer, &span);
                let node = builder.add_literal(span.clone(), idx, InternedKind::HexInt);
                (node, span)
            }
            TokenKind::Float => {
                let idx = intern(session.clone(), lexer, &span);
                let node = builder.add_literal(span.clone(), idx, InternedKind::Float);
                (node, span)
            }
            TokenKind::IntFloat => {
                let idx = intern(session.clone(), lexer, &span);
                let node = builder.add_literal(span.clone(), idx, InternedKind::IntFloat);
                (node, span)
            }
            TokenKind::FloatExp => {
                let idx = intern(session.clone(), lexer, &span);
                let node = builder.add_literal(span.clone(), idx, InternedKind::FloatExp);
                (node, span)
            }
            TokenKind::IntExp => {
                let idx = intern(session.clone(), lexer, &span);
                let node = builder.add_literal(span.clone(), idx, InternedKind::IntExp);
                (node, span)
            }
            TokenKind::String => {
                let idx = intern(session.clone(), lexer, &span);
                let node = builder.add_literal(span.clone(), idx, InternedKind::Str);
                (node, span)
            }
            TokenKind::RawString => {
                let idx = intern(session.clone(), lexer, &span);
                let node = builder.add_literal(span.clone(), idx, InternedKind::RawStr);
                (node, span)
            }
            TokenKind::Char => {
                let idx = intern(session.clone(), lexer, &span);
                let node = builder.add_literal(span.clone(), idx, InternedKind::Char);
                (node, span)
            }
            TokenKind::Identifier => continue_path(session.clone(), lexer, builder, span)?,
            TokenKind::Minus => {
                prefix_op!(session.clone(), lexer, builder, span, core::ops::Neg::neg)
            }
            TokenKind::Star => {
                prefix_op!(session.clone(), lexer, builder, span, core::ops::Deref::deref)
            }
            TokenKind::Bang => {
                prefix_op!(session.clone(), lexer, builder, span, core::ops::LogicalNot::not)
            }
            TokenKind::Tilde => {
                prefix_op!(session.clone(), lexer, builder, span, core::ops::BitNot::not)
            }
            TokenKind::DoublePlus => {
                prefix_op!(session.clone(), lexer, builder, span, core::ops::PreInc::inc)
            }
            TokenKind::DoubleMinus => {
                prefix_op!(session.clone(), lexer, builder, span, core::ops::PreDec::dec)
            }
            kind => return Err(ParserError::InvalidPrefixOperator { found: kind, span }),
        },
        None => return Err(ParserError::UnexpectedEof),
    };

    loop {
        lhs = match peek(lexer)? {
            Some(Token { kind: TokenKind::DoublePlus, .. }) => {
                postfix_op!(session.clone(), lexer, builder, lhs, 11, core::ops::PostInc::inc)
            }
            Some(Token { kind: TokenKind::DoubleMinus, .. }) => {
                postfix_op!(session.clone(), lexer, builder, lhs, 11, core::ops::PostDec::dec)
            }
            Some(Token { kind: TokenKind::Question, .. }) => {
                let Token { span: lhs_span, .. } = unsafe { next(lexer)?.unwrap_unchecked() };
                if 11 < min_bp {
                    break;
                }

                let mut lexer_clone = lexer.clone();
                if let Ok((expr1, expr1_span)) = builder.checkpoint().resolve(parse_expr(
                    session.clone(),
                    &mut lexer_clone,
                    builder,
                    0,
                )) {
                    *lexer = lexer_clone;
                    match peek(lexer)? {
                        Some(Token { kind: TokenKind::Colon, .. }) => {
                            let _ = unsafe { next(lexer)?.unwrap_unchecked() };
                            lexer_clone = lexer.clone();

                            let (expr2, expr2_span) =
                                parse_expr(session.clone(), &mut lexer_clone, builder, 0)?;

                            *lexer = lexer_clone;
                            let span = lhs.1.join(&expr2_span);
                            let node = builder.add_if_expr(span.clone(), lhs.0, expr1, Some(expr2));
                            (node, span)
                        }
                        _ => {
                            let some_path = crate::path::create_path(
                                (session.clone()),
                                builder,
                                [
                                    ("core", lhs.1.clone()),
                                    ("option", lhs.1.clone()),
                                    ("Option", lhs.1.clone()),
                                    ("Some", lhs.1.clone()),
                                ],
                            );
                            let none_path = crate::path::create_path(
                                (session.clone()),
                                builder,
                                [
                                    ("core", lhs.1.clone()),
                                    ("option", lhs.1.clone()),
                                    ("Option", lhs.1.clone()),
                                    ("None", lhs.1.clone()),
                                ],
                            );
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
                    let path = crate::path::create_path(
                        (session.clone()),
                        builder,
                        [
                            ("core", lhs.1.clone()),
                            ("ops", lhs.1.clone()),
                            ("Try", lhs.1.clone()),
                            ("unwrap", lhs.1.clone()),
                        ],
                    );
                    let span = lhs.1.join(&lhs_span);
                    let node = builder.add_fn_call(span.clone(), path, lhs.0);
                    (node, span)
                }
            }
            Some(Token { kind: TokenKind::Plus, .. }) => {
                infix_op!(session.clone(), lexer, builder, lhs, 5, 6, core::ops::Addition::add)
            }
            Some(Token { kind: TokenKind::Minus, .. }) => {
                infix_op!(session.clone(), lexer, builder, lhs, 5, 6, core::ops::Subtraction::sub)
            }
            Some(Token { kind: TokenKind::Star, .. }) => {
                infix_op!(
                    session.clone(),
                    lexer,
                    builder,
                    lhs,
                    7,
                    8,
                    core::ops::Multiplication::mul
                )
            }
            Some(Token { kind: TokenKind::Slash, .. }) => {
                infix_op!(session.clone(), lexer, builder, lhs, 7, 8, core::ops::Division::div)
            }
            Some(Token { kind: TokenKind::Percent, .. }) => {
                infix_op!(session.clone(), lexer, builder, lhs, 7, 8, core::ops::Reminder::rem)
            }
            Some(Token { kind: TokenKind::Ampersand, .. }) => {
                infix_op!(session.clone(), lexer, builder, lhs, 3, 4, core::ops::BitAnd::and)
            }
            Some(Token { kind: TokenKind::Pipe, .. }) => {
                infix_op!(session.clone(), lexer, builder, lhs, 1, 2, core::ops::BitOr::or)
            }
            Some(Token { kind: TokenKind::Caret, .. }) => {
                infix_op!(session.clone(), lexer, builder, lhs, 2, 3, core::ops::BitXor::xor)
            }
            Some(Token { kind: TokenKind::LShift, .. }) => {
                infix_op!(session.clone(), lexer, builder, lhs, 6, 7, core::ops::Shl::shl)
            }
            Some(Token { kind: TokenKind::RShift, .. }) => {
                infix_op!(session.clone(), lexer, builder, lhs, 6, 7, core::ops::Shr::shr)
            }
            Some(Token { kind: TokenKind::ARShift, .. }) => {
                infix_op!(session.clone(), lexer, builder, lhs, 6, 7, core::ops::Shr::arshr)
            }
            Some(Token { kind: TokenKind::DoubleAmpersand, .. }) => {
                infix_op!(session.clone(), lexer, builder, lhs, 9, 10, core::ops::LogicalAnd::and)
            }
            Some(Token { kind: TokenKind::DoublePipe, .. }) => {
                infix_op!(session.clone(), lexer, builder, lhs, 9, 10, core::ops::LogicalOr::or)
            }
            Some(Token { kind: TokenKind::DoubleEqual, .. }) => {
                infix_op!(session.clone(), lexer, builder, lhs, 4, 5, core::ops::Eq::eq)
            }
            Some(Token { kind: TokenKind::NotEqual, .. }) => {
                infix_op!(session.clone(), lexer, builder, lhs, 4, 5, core::ops::Eq::ne)
            }
            Some(Token { kind: TokenKind::Less, .. }) => {
                infix_op!(session.clone(), lexer, builder, lhs, 4, 5, core::ops::Ord::lt)
            }
            Some(Token { kind: TokenKind::LessEqual, .. }) => {
                infix_op!(session.clone(), lexer, builder, lhs, 4, 5, core::ops::Ord::le)
            }
            Some(Token { kind: TokenKind::Greater, .. }) => {
                infix_op!(session.clone(), lexer, builder, lhs, 4, 5, core::ops::Ord::gt)
            }
            Some(Token { kind: TokenKind::GreaterEqual, .. }) => {
                infix_op!(session.clone(), lexer, builder, lhs, 4, 5, core::ops::Ord::ge)
            }
            Some(Token { kind: TokenKind::DoubleDot, .. }) => {
                infix_op!(session.clone(), lexer, builder, lhs, 11, 11, core::ops::Range::new)
            }
            None | Some(_) => break,
        };
    }

    Ok(lhs)
}

pub fn parse_block<'src>(
    session: Arc<Mutex<Session>>,
    lexer: &mut Lexer<'src>,
    builder: &mut AstBuilder,
) -> ParseResult {
    let start = expect(lexer, TokenKind::LBrace)?.span.start;
    // TODO: parse statements
    let end = expect(lexer, TokenKind::RBrace)?.span.end;

    let span = Span::new(start, end);
    let node = builder.add_block(span.clone(), None);
    Ok((node, span))
}
