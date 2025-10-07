use osta_ast::ast::{Interned, InternedKind, Ty, TyKind};
use osta_ast::{AstBuilder, NodeId, AST};
use osta_lexer::{Lexer, LexerError, Token, TokenKind};
use osta_session::interner::InternId;
use osta_session::Session;
use osta_syntax::Span;
use std::sync::{Arc, Mutex};
use thiserror::Error;

pub struct Parser<'src> {
    session: Arc<Mutex<Session>>,
    lexer: Lexer<'src>,
    builder: AstBuilder,
}

pub fn parse(session: Arc<Mutex<Session>>, source: &str) -> Result<AST, ParserError> {
    let mut parser = Parser::new(session, source);
    parser.parse_root()?;
    Ok(parser.builder.build())
}

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
}

pub type ParseResult<T = (NodeId, Span)> = Result<T, ParserError>;

impl<'src> Parser<'src> {
    pub fn new(session: Arc<Mutex<Session>>, source: &'src str) -> Self {
        Self {
            session,
            lexer: Lexer::new(source),
            builder: AstBuilder::new(),
        }
    }

    pub fn parse_root(&mut self) -> ParseResult<()> {
        while self.peek(0)?.is_some() {
            let id = self.builder.checkpoint().resolve(self.parse_item())?;
            self.builder.add_item(id);
        }
        Ok(())
    }

    pub fn parse_item(&mut self) -> ParseResult<NodeId> {
        Ok(self.parse_fn_decl()?.0)
    }

    pub fn parse_fn_decl(&mut self) -> ParseResult {
        let start = self.expect(TokenKind::Fn)?.span.start;
        let ident = self.parse_ident()?;
        let _ = self.expect(TokenKind::LParen)?;
        let _ = self.expect(TokenKind::RParen)?;
        let _ = self.expect(TokenKind::Arrow)?;
        let ty = self.parse_type()?.0;
        let (body, block_span) = self.parse_block()?;
        let end = block_span.end;

        let span = Span::new(start, end);
        let node = self
            .builder
            .add_fn_decl(span.clone(), ident, Some(ty), body);

        Ok((node, span))
    }

    pub fn parse_type(&mut self) -> ParseResult<(Ty, Span)> {
        let (ty, span) = match self.next()? {
            Some(Token { kind: TokenKind::Never, span }) => (TyKind::Never, span),
            Some(Token { kind: TokenKind::Void, span }) => (TyKind::Void, span),
            Some(Token { kind: TokenKind::UintType(size), span }) => (TyKind::Uint(size), span),
            Some(Token { kind: TokenKind::IntType(size), span }) => (TyKind::Int(size), span),
            Some(Token { kind: TokenKind::FloatType(size), span }) => (TyKind::Float(size), span),
            Some(Token { kind: TokenKind::Identifier, span }) => {
                let (path_id, span) = self.continue_path(span)?;
                (TyKind::Other(path_id), span)
            }
            Some(Token { kind, span }) => {
                return Err(ParserError::InvalidType { found: kind, span });
            }
            None => return Err(ParserError::LexerError(LexerError::UnexpectedEof)),
        };
        let ty = Ty { span: span.clone(), kind: ty };
        Ok((ty, span))
    }

    pub fn parse_path(&mut self, root: bool) -> ParseResult {
        let ident = match self.next()? {
            Some(Token { kind: TokenKind::Identifier | TokenKind::Super, span }) => {
                let idx = self.intern(&span);
                Interned::new(idx, span, InternedKind::Ident)
            }
            Some(Token { kind: TokenKind::Package, span }) if root => {
                let idx = self.intern(&span);
                Interned::new(idx, span, InternedKind::Ident)
            }
            Some(Token { kind: TokenKind::MacroIdentifier, span }) => {
                let idx = self.intern(&span);
                let ident = Interned::new(idx, span.clone(), InternedKind::Ident);
                let node = self.builder.add_path(span.clone(), ident, None);
                return Ok((node, span));
            }
            Some(Token { kind, span }) => {
                return Err(ParserError::InvalidPath { found: kind, span });
            }
            None => return Err(ParserError::LexerError(LexerError::UnexpectedEof)),
        };
        match self.expect_opt(TokenKind::DoubleColon)? {
            Some(_) => {
                let next = self.parse_path(false)?.0;
                let span = ident.span.join(&self.builder.span_of(next));
                let node = self.builder.add_path(span.clone(), ident, Some(next));
                Ok((node, span))
            }
            None => {
                let span = ident.span.clone();
                let node = self.builder.add_path(span.clone(), ident, None);
                Ok((node, span))
            }
        }
    }

    fn continue_path(&mut self, span: Span) -> ParseResult {
        match self.expect_opt(TokenKind::DoubleColon)? {
            Some(_) => {
                let (path_id, path_span) = self.parse_path(false)?;
                Ok((path_id, span.join(&path_span)))
            }
            _ => {
                let idx = self.intern(&span);
                let ident = Interned::new(idx, span.clone(), InternedKind::Ident);
                let path_id = self.builder.add_path(span.clone(), ident, None);
                Ok((path_id, span))
            }
        }
    }

    pub fn parse_block(&mut self) -> ParseResult {
        let start = self.expect(TokenKind::LBrace)?.span.start;
        // TODO: parse statements
        let end = self.expect(TokenKind::RBrace)?.span.end;

        let span = Span::new(start, end);
        let node = self.builder.add_block(span.clone(), None);
        Ok((node, span))
    }

    pub fn parse_expr(&mut self, min_bp: u8) -> ParseResult {
        macro_rules! prefix_op {
            ($op_span:ident, $($path:ident)::+) => {{
                let (rhs, rhs_span) = self.parse_expr(9)?;
                let path = self.create_path([
                    $((stringify!($path), $op_span.clone())),+
                ]);
                let span = $op_span.join(&rhs_span);
                let node = self.builder.add_fn_call(span.clone(), path, rhs);
                (node, span)
            }};
        }

        macro_rules! postfix_op {
            ($lhs:ident, $l_bp:literal, $($path:ident)::+) => {{
                let Token { span, .. } = unsafe { self.next()?.unwrap_unchecked() };
                if $l_bp < min_bp {
                    break;
                }
                let path = self.create_path([
                    $((stringify!($path), $lhs.1.clone())),+
                ]);
                let span = $lhs.1.join(&span);
                let node = self.builder.add_fn_call(span.clone(), path, $lhs.0);
                (node, span)
            }};
        }

        macro_rules! infix_op {
            ($lhs:ident, $l_bp:literal, $r_bp:literal, $($path:ident)::+) => {{
                let Token { span, .. } = unsafe { self.next()?.unwrap_unchecked() };
                if $l_bp < min_bp {
                    break;
                } else if $l_bp == min_bp {
                    return Err(ParserError::AmbiguousOperator { span });
                }
                let (rhs, rhs_span) = self.parse_expr($r_bp)?;
                let path = self.create_path([
                    $((stringify!($path), span.clone())),+
                ]);
                let span = $lhs.1.join(&rhs_span);
                let args = self.builder.add_chain(span.clone(), $lhs.0, rhs);
                let node = self.builder.add_fn_call(span.clone(), path, args);
                (node, span)
            }};
        }

        let mut lhs = match self.next()? {
            Some(Token { kind: TokenKind::LParen, span: lparen_span }) => {
                let lhs = self.parse_expr(0)?.0;
                let rparen_span = self.expect(TokenKind::RParen)?.span;
                let span = lparen_span.join(&rparen_span);
                (lhs, span)
            }
            Some(Token { kind, span }) => match kind {
                TokenKind::DecInt => {
                    let idx = self.intern(&span);
                    let node = self
                        .builder
                        .add_literal(span.clone(), idx, InternedKind::DecInt);
                    (node, span)
                }
                TokenKind::BinInt => {
                    let idx = self.intern(&span);
                    let node = self
                        .builder
                        .add_literal(span.clone(), idx, InternedKind::BinInt);
                    (node, span)
                }
                TokenKind::OctInt => {
                    let idx = self.intern(&span);
                    let node = self
                        .builder
                        .add_literal(span.clone(), idx, InternedKind::OctInt);
                    (node, span)
                }
                TokenKind::HexInt => {
                    let idx = self.intern(&span);
                    let node = self
                        .builder
                        .add_literal(span.clone(), idx, InternedKind::HexInt);
                    (node, span)
                }
                TokenKind::Float => {
                    let idx = self.intern(&span);
                    let node = self
                        .builder
                        .add_literal(span.clone(), idx, InternedKind::Float);
                    (node, span)
                }
                TokenKind::IntFloat => {
                    let idx = self.intern(&span);
                    let node = self
                        .builder
                        .add_literal(span.clone(), idx, InternedKind::IntFloat);
                    (node, span)
                }
                TokenKind::FloatExp => {
                    let idx = self.intern(&span);
                    let node = self
                        .builder
                        .add_literal(span.clone(), idx, InternedKind::FloatExp);
                    (node, span)
                }
                TokenKind::IntExp => {
                    let idx = self.intern(&span);
                    let node = self
                        .builder
                        .add_literal(span.clone(), idx, InternedKind::IntExp);
                    (node, span)
                }
                TokenKind::String => {
                    let idx = self.intern(&span);
                    let node = self
                        .builder
                        .add_literal(span.clone(), idx, InternedKind::Str);
                    (node, span)
                }
                TokenKind::RawString => {
                    let idx = self.intern(&span);
                    let node = self
                        .builder
                        .add_literal(span.clone(), idx, InternedKind::RawStr);
                    (node, span)
                }
                TokenKind::Char => {
                    let idx = self.intern(&span);
                    let node = self
                        .builder
                        .add_literal(span.clone(), idx, InternedKind::Char);
                    (node, span)
                }
                TokenKind::Identifier => self.continue_path(span)?,
                TokenKind::Minus => prefix_op!(span, core::ops::Neg::neg),
                TokenKind::Star => prefix_op!(span, core::ops::Deref::deref),
                TokenKind::Bang => prefix_op!(span, core::ops::LogicalNot::not),
                TokenKind::Tilde => prefix_op!(span, core::ops::BitNot::not),
                TokenKind::DoublePlus => prefix_op!(span, core::ops::PreInc::inc),
                TokenKind::DoubleMinus => prefix_op!(span, core::ops::PreDec::dec),
                kind => return Err(ParserError::InvalidPrefixOperator { found: kind, span }),
            },
            None => return Err(ParserError::LexerError(LexerError::UnexpectedEof)),
        };

        loop {
            lhs = match self.peek(0)? {
                Some(Token { kind: TokenKind::DoublePlus, .. }) => {
                    postfix_op!(lhs, 11, core::ops::PostInc::inc)
                }
                Some(Token { kind: TokenKind::DoubleMinus, .. }) => {
                    postfix_op!(lhs, 11, core::ops::PostDec::dec)
                }
                Some(Token { kind: TokenKind::Question, .. }) => {
                    // TODO: Option creation (bool ? expr -> if bool Some(expr) else None)
                    // TODO: Ternary operator (bool ? expr1 : expr2 -> if bool expr1 else expr2)
                    postfix_op!(lhs, 11, core::ops::Try::unwrap)
                }
                Some(Token { kind: TokenKind::Plus, .. }) => {
                    infix_op!(lhs, 5, 6, core::ops::Addition::add)
                }
                Some(Token { kind: TokenKind::Minus, .. }) => {
                    infix_op!(lhs, 5, 6, core::ops::Subtraction::sub)
                }
                Some(Token { kind: TokenKind::Star, .. }) => {
                    infix_op!(lhs, 7, 8, core::ops::Multiplication::mul)
                }
                Some(Token { kind: TokenKind::Slash, .. }) => {
                    infix_op!(lhs, 7, 8, core::ops::Division::div)
                }
                Some(Token { kind: TokenKind::Percent, .. }) => {
                    infix_op!(lhs, 7, 8, core::ops::Reminder::rem)
                }
                Some(Token { kind: TokenKind::Ampersand, .. }) => {
                    infix_op!(lhs, 3, 4, core::ops::BitAnd::and)
                }
                Some(Token { kind: TokenKind::Pipe, .. }) => {
                    infix_op!(lhs, 1, 2, core::ops::BitOr::or)
                }
                Some(Token { kind: TokenKind::Caret, .. }) => {
                    infix_op!(lhs, 2, 3, core::ops::BitXor::xor)
                }
                Some(Token { kind: TokenKind::LShift, .. }) => {
                    infix_op!(lhs, 6, 7, core::ops::Shl::shl)
                }
                Some(Token { kind: TokenKind::RShift, .. }) => {
                    infix_op!(lhs, 6, 7, core::ops::Shr::shr)
                }
                Some(Token { kind: TokenKind::ARShift, .. }) => {
                    infix_op!(lhs, 6, 7, core::ops::Shr::arshr)
                }
                Some(Token { kind: TokenKind::DoubleAmpersand, .. }) => {
                    infix_op!(lhs, 9, 10, core::ops::LogicalAnd::and)
                }
                Some(Token { kind: TokenKind::DoublePipe, .. }) => {
                    infix_op!(lhs, 9, 10, core::ops::LogicalOr::or)
                }
                Some(Token { kind: TokenKind::DoubleEqual, .. }) => {
                    infix_op!(lhs, 4, 5, core::ops::Eq::eq)
                }
                Some(Token { kind: TokenKind::NotEqual, .. }) => {
                    infix_op!(lhs, 4, 5, core::ops::Eq::ne)
                }
                Some(Token { kind: TokenKind::Less, .. }) => {
                    infix_op!(lhs, 4, 5, core::ops::Ord::lt)
                }
                Some(Token { kind: TokenKind::LessEqual, .. }) => {
                    infix_op!(lhs, 4, 5, core::ops::Ord::le)
                }
                Some(Token { kind: TokenKind::Greater, .. }) => {
                    infix_op!(lhs, 4, 5, core::ops::Ord::gt)
                }
                Some(Token { kind: TokenKind::GreaterEqual, .. }) => {
                    infix_op!(lhs, 4, 5, core::ops::Ord::ge)
                }
                Some(Token { kind: TokenKind::DoubleDot, .. }) => {
                    infix_op!(lhs, 11, 11, core::ops::Range::new)
                }
                None | Some(_) => break,
            };
        }

        Ok(lhs)
    }

    pub fn parse_ident(&mut self) -> ParseResult<Interned> {
        let tok = self.expect(TokenKind::Identifier)?;
        let idx = self.intern(&tok.span);

        Ok(Interned::new(idx, tok.span, InternedKind::Ident))
    }

    fn next(&mut self) -> ParseResult<Option<Token>> {
        match self.lexer.next() {
            Some(Ok(tok)) => Ok(Some(tok)),
            Some(Err(e)) => Err(ParserError::LexerError(e)),
            None => Ok(None),
        }
    }

    fn expect(&mut self, kind: TokenKind) -> ParseResult<Token> {
        match self.lexer.expect(kind) {
            Ok(tok) => Ok(tok),
            Err(e) => Err(ParserError::LexerError(e)),
        }
    }

    fn expect_opt(&mut self, kind: TokenKind) -> ParseResult<Option<Token>> {
        match self.lexer.expect_opt(kind) {
            Some(Ok(tok)) => Ok(Some(tok)),
            Some(Err(e)) => Err(ParserError::LexerError(e)),
            None => Ok(None),
        }
    }

    fn peek<'a>(&'a mut self, n: usize) -> ParseResult<Option<&'a Token>> {
        match self.lexer.peek(n) {
            Some(Ok(tok)) => Ok(Some(unsafe { &*(tok as *const Token) as &'a Token })),
            Some(Err(_)) => match self.lexer.next() {
                Some(Err(e)) => Err(ParserError::LexerError(e)),
                _ => unreachable!("unreachable due to lexer error"),
            },
            None => Ok(None),
        }
    }

    // Alternative peek implementation that avoids unsafe, but calls peek multiple times
    // which may have performance implications depending on the lexer implementation.
    //
    // fn peek(&mut self, n: usize) -> ParseResult<Option<&Token>> {
    //     // Check if it's a successful peek without storing the ref
    //     if matches!(self.lexer.peek(n), Some(Ok(_))) {
    //         // Now safely peek again to get and return the ref
    //         if let Some(Ok(tok)) = self.lexer.peek(n) {
    //             return Ok(Some(tok));
    //         } else {
    //             unreachable!("Peek changed unexpectedly");
    //         }
    //     }
    //
    //     // For error or none, peek again and handle mutation if needed
    //     match self.lexer.peek(n) {
    //         Some(Err(_)) => match self.lexer.next() {
    //             Some(Err(e)) => Err(ParserError::LexerError(e)),
    //             _ => unreachable!("Lexer peek showed Err, but next didn't"),
    //         },
    //         None => Ok(None),
    //         _ => unreachable!("Peek changed unexpectedly"),
    //     }
    // }

    fn intern(&mut self, span: &Span) -> InternId {
        let s = span.slice(self.lexer.source());
        let mut session = self.session.lock().unwrap();
        session.interner.get_or_intern(s)
    }

    fn intern_str(&mut self, s: &str) -> InternId {
        let mut session = self.session.lock().unwrap();
        session.interner.get_or_intern(s)
    }

    fn create_path<const N: usize>(&mut self, parts: [(&str, Span); N]) -> NodeId {
        let mut current: Option<NodeId> = None;
        for (part, span) in parts.into_iter().rev() {
            let idx = self.intern_str(part);
            let ident = Interned::new(idx, span, InternedKind::Ident);
            let span = if let Some(next) = current {
                let next_span = self.builder.span_of(next);
                ident.span.join(&next_span)
            } else {
                ident.span.clone()
            };
            current = Some(self.builder.add_path(span, ident, current));
        }
        current.unwrap()
    }
}
