use crate::util::{advance_if, intern, next};
use crate::{err, expect_one_of, ParseResult, ParserError};
use osta_ast::ast::{Interned, InternedKind};
use osta_ast::{AstBuilder, NodeId};
use osta_lexer::{Lexer, Token, TokenKind};
use osta_session::Session;
use osta_syntax::Span;

pub fn parse_ident(session: &mut Session, lexer: &mut Lexer) -> ParseResult<Interned> {
    let tok = expect_one_of!(
        lexer,
        TokenKind::Identifier | TokenKind::MacroIdentifier | TokenKind::ComptimeIdentifier
    )?;
    let idx = intern(session, lexer, &tok.span);

    Ok(Interned::new(idx, tok.span, InternedKind::Ident))
}

pub fn parse_path<'src>(
    session: &mut Session,
    lexer: &mut Lexer<'src>,
    builder: &mut AstBuilder,
    root: bool,
) -> ParseResult {
    let ident = match next(lexer)? {
        Some(Token { kind: TokenKind::Identifier | TokenKind::Super, span }) => {
            let idx = intern(session, lexer, &span);
            Interned::new(idx, span, InternedKind::Ident)
        }
        Some(Token { kind: TokenKind::Package, span }) if root => {
            let idx = intern(session, lexer, &span);
            Interned::new(idx, span, InternedKind::Ident)
        }
        Some(Token { kind: TokenKind::MacroIdentifier, span }) => {
            let idx = intern(session, lexer, &span);
            let ident = Interned::new(idx, span.clone(), InternedKind::Ident);
            let node = builder.add_path(span.clone(), ident, None);
            return Ok((node, span));
        }
        Some(Token { kind, span }) => {
            return err!(ParserError::InvalidPath { found: kind, span });
        }
        None => return err!(ParserError::UnexpectedEof),
    };
    if advance_if(lexer, TokenKind::DoubleColon)? {
        let next = parse_path(session, lexer, builder, false)?.0;
        let span = ident.span.join(&builder.span_of(next));
        let node = builder.add_path(span.clone(), ident, Some(next));
        Ok((node, span))
    } else {
        let span = ident.span.clone();
        let node = builder.add_path(span.clone(), ident, None);
        Ok((node, span))
    }
}

pub fn continue_path<'src>(
    session: &mut Session,
    lexer: &mut Lexer<'src>,
    builder: &mut AstBuilder,
    span: Span,
) -> ParseResult {
    if advance_if(lexer, TokenKind::DoubleColon)? {
        let (path_id, path_span) = parse_path(session, lexer, builder, false)?;
        Ok((path_id, span.join(&path_span)))
    } else {
        let idx = intern(session, lexer, &span);
        let ident = Interned::new(idx, span.clone(), InternedKind::Ident);
        let path_id = builder.add_path(span.clone(), ident, None);
        Ok((path_id, span))
    }
}

pub fn create_path<const N: usize>(
    session: &mut Session,
    builder: &mut AstBuilder,
    parts: [(&str, Span); N],
) -> NodeId {
    let mut current: Option<NodeId> = None;
    for (part, span) in parts.into_iter().rev() {
        let idx = session.get_or_intern(part);
        let ident = Interned::new(idx, span, InternedKind::Ident);
        let span = if let Some(next) = current {
            let next_span = builder.span_of(next);
            ident.span.join(&next_span)
        } else {
            ident.span.clone()
        };
        current = Some(builder.add_path(span, ident, current));
    }
    current.unwrap()
}
