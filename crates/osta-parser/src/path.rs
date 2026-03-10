use crate::util::{advance_if, next};
use crate::{err, expect_choice, FileSession, ParseResult, ParserErrorKind};
use osta_ast::ast::{Interned, InternedKind};
use osta_ast::NodeId;
use osta_lexer::{Token, TokenKind};
use osta_syntax::Span;

pub fn parse_ident() -> ParseResult<Interned> {
    let tok = expect_choice!(
        TokenKind::Identifier | TokenKind::MacroIdentifier | TokenKind::ComptimeIdentifier
    )?;
    let idx = FileSession::intern(&tok.span);

    Ok(Interned::new(idx, tok.span, InternedKind::Ident))
}

pub fn parse_path<'src>(root: bool) -> ParseResult {
    let builder = FileSession::ast();

    let ident = match next()? {
        Some(Token { kind: TokenKind::Identifier | TokenKind::Super, span }) => {
            let idx = FileSession::intern(&span);
            Interned::new(idx, span, InternedKind::Ident)
        }
        Some(Token { kind: TokenKind::Package, span }) if root => {
            let idx = FileSession::intern(&span);
            Interned::new(idx, span, InternedKind::Ident)
        }
        Some(Token { kind: TokenKind::MacroIdentifier, span }) => {
            let idx = FileSession::intern(&span);
            let ident = Interned::new(idx, span.clone(), InternedKind::Ident);
            let node = builder.add_path(span.clone(), ident, None);
            return Ok((node, span));
        }
        Some(Token { kind, span }) => {
            return err!(ParserErrorKind::UnexpectedToken { found: kind, span });
        }
        None => return err!(ParserErrorKind::UnexpectedEof),
    };
    if advance_if(TokenKind::DoubleColon)? {
        let next = parse_path(false)?.0;
        let span = ident.span.join(&builder.span_of(next));
        let node = builder.add_path(span.clone(), ident, Some(next));
        Ok((node, span))
    } else {
        let span = ident.span.clone();
        let node = builder.add_path(span.clone(), ident, None);
        Ok((node, span))
    }
}

pub fn continue_path<'src>(span: Span) -> ParseResult {
    let builder = FileSession::ast();

    if advance_if(TokenKind::DoubleColon)? {
        let (path_id, path_span) = parse_path(false)?;
        Ok((path_id, span.join(&path_span)))
    } else {
        let idx = FileSession::intern(&span);
        let ident = Interned::new(idx, span.clone(), InternedKind::Ident);
        let path_id = builder.add_path(span.clone(), ident, None);
        Ok((path_id, span))
    }
}

pub fn create_path<const N: usize>(parts: [(&str, Span); N]) -> NodeId {
    let builder = FileSession::ast();

    let mut current: Option<NodeId> = None;
    for (part, span) in parts.into_iter().rev() {
        let idx = FileSession::intern_str(part);
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
