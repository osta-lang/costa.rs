use crate::error::{ParserErrorKind, ParserIntent};
use crate::expr::{parse_block, parse_expr};
use crate::path::{continue_path, create_path, parse_ident};
use crate::util::{advance_if, expect, next, peek};
use crate::{err, scoped_intent, try_parse, FileSession, ParseResult};
use osta_ast::ast::Ty;
use osta_ast::NodeId;
use osta_lexer::{Token, TokenKind};
use osta_syntax::Span;

pub fn parse_item<'src>() -> ParseResult<NodeId> {
    Ok(parse_fn_decl()?.0)
}

pub fn parse_fn_decl<'src>() -> ParseResult {
    scoped_intent!(ParserIntent::FuncDecl);

    let builder = FileSession::ast();

    let start = expect(TokenKind::Fn)?.span.start;
    let ident = parse_ident()?;
    let _ = expect(TokenKind::LParen)?;
    let args = try_parse!(parse_fn_decl_args).ok().map(|(id, _)| id);
    let _ = expect(TokenKind::RParen)?;
    let ty = if advance_if(TokenKind::Arrow)? {
        let ty = parse_type()?.0;
        Some(ty)
    } else {
        None
    };
    let (body, block_span) = parse_block()?;
    let end = block_span.end;

    let span = Span::new(start, end);
    let node = builder.add_fn_decl(span.clone(), ident, args, ty, body);

    Ok((node, span))
}

pub fn parse_fn_decl_args() -> ParseResult {
    let builder = FileSession::ast();

    let (this_id, this_span) = {
        let ident = parse_ident()?;
        expect(TokenKind::Colon)?;
        let ty = parse_type()?;

        let span = ident.span.join(&ty.1);
        let id = builder.add_variable_binding(span.clone(), ident, Some(ty.0), None);
        (id, span)
    };
    match try_parse!(parse_fn_decl_args) {
        Ok((next_id, next_span)) => {
            let span = this_span.join(&next_span);
            let node_id = builder.add_chain(span.clone(), this_id, next_id);
            Ok((node_id, span))
        }
        _ => Ok((this_id, this_span)),
    }
}

pub fn parse_type<'src>() -> ParseResult {
    let builder = FileSession::ast();

    let ty = match next()? {
        Some(Token { kind: TokenKind::Never, span }) => {
            (builder.add_type(span.clone(), Ty::Never), span)
        }
        Some(Token { kind: TokenKind::Void, span }) => {
            (builder.add_type(span.clone(), Ty::Void), span)
        }
        Some(Token { kind: TokenKind::UintType(size), span }) => {
            (builder.add_type(span.clone(), Ty::Uint(size)), span)
        }
        Some(Token { kind: TokenKind::IntType(size), span }) => {
            (builder.add_type(span.clone(), Ty::Int(size)), span)
        }
        Some(Token { kind: TokenKind::FloatType(size), span }) => {
            (builder.add_type(span.clone(), Ty::Float(size)), span)
        }
        Some(Token { kind: TokenKind::Identifier, span }) => {
            let (path_id, span) = continue_path(span)?;
            (builder.add_type(span.clone(), Ty::Path(path_id)), span)
        }
        Some(Token { kind: TokenKind::Ampersand, span }) => {
            let (ty, span_next) = parse_type()?;
            (
                builder.add_type(span.clone(), Ty::Reference(ty)),
                Span::new(span.start, span_next.end),
            )
        }
        Some(Token { kind: TokenKind::Star, span }) => {
            let (ty, span_next) = parse_type()?;
            (
                builder.add_type(span.clone(), Ty::Pointer(ty)),
                Span::new(span.start, span_next.end),
            )
        }
        Some(Token { kind: TokenKind::LBracket, span }) => {
            let (ty, _) = parse_type()?;

            let count = if let Some(Token { kind: TokenKind::Semicolon, .. }) = peek()? {
                next()?;
                let (number, _) = parse_expr(0)?;
                Some(number)
            } else {
                None
            };

            let terminator = if let Some(Token { kind: TokenKind::Colon, .. }) = peek()? {
                next()?;
                let (terminator, _) = parse_expr(0)?;
                Some(terminator)
            } else {
                None
            };

            let Token { span: rb_span, .. } = expect(TokenKind::RBracket)?;

            (
                builder.add_type(span.clone(), Ty::Array { ty, count, terminator }),
                Span::new(span.start, rb_span.end),
            )
        }
        Some(Token { kind: TokenKind::Type, span }) => {
            let path = create_path([
                ("std", span.clone()),
                ("comptime", span.clone()),
                ("Type", span.clone()),
            ]);
            (builder.add_type(span.clone(), Ty::Path(path)), span)
        }
        Some(Token { kind, span }) => {
            return err!(ParserErrorKind::UnexpectedToken { found: kind, span });
        }
        None => return err!(ParserErrorKind::UnexpectedEof),
    };

    Ok(ty)
}
