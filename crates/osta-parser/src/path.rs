use crate::error::ParserErrorPolicy;
use crate::util::{advance_if, next};
use crate::{err, expect_choice, FileSession, ParseResult};
use miette::{miette, LabeledSpan, Severity};
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

pub fn parse_path(root: bool) -> ParseResult {
    let builder = FileSession::builder();

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
            let msg = match kind {
                TokenKind::Package if !root => (
                    "`package` can only be used at the root of a path".to_string(),
                    "`package` can only be used at the root of a path".to_string(),
                ),
                _ => (
                    format!("Path must start by an identifier, `super` or `package`, but `{kind:?}` was found"),
                    format!("Unexpected token! `{kind:?}` was found, but an identifier, `super` or `package` was expected"),
                ),
            };

            return err!(
                miette! {
                    severity = Severity::Error,
                    code = "parser/path",
                    labels = vec![LabeledSpan::new(
                        Some(msg.0),
                        span.start,
                        span.end - span.start,
                    )],
                    "{}", msg.1
                },
                ParserErrorPolicy::TryOthers
            );
        }
        None => {
            return err!(
                miette! {
                    severity = Severity::Error,
                    code = "parser/path/eof",
                    labels = vec![LabeledSpan::new(
                        Some("An identifier, `super` or `package` was expected".to_string()),
                        FileSession::lexer().source().len(),
                        0
                    )],
                    "Unexpected end of file! An identifier, `super` or `package` was expected"
                },
                ParserErrorPolicy::Undefined
            )
        }
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

pub fn continue_path(span: Span) -> ParseResult {
    let builder = FileSession::builder();

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
    let builder = FileSession::builder();

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
