use osta_ast::ast::{Ident, Ty, TyKind};
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
}

pub type ParseResult<T = NodeId> = Result<T, ParserError>;

impl<'src> Parser<'src> {
    pub fn new(session: Arc<Mutex<Session>>, source: &'src str) -> Self {
        Self {
            session,
            lexer: Lexer::new(source),
            builder: AstBuilder::new(),
        }
    }

    pub fn parse_root(&mut self) -> ParseResult<()> {
        while let Some(Ok(_)) = self.lexer.peek(0) {
            let id = self.builder.checkpoint().resolve(self.parse_item())?;
            self.builder.add_item(id);
        }
        Ok(())
    }

    pub fn parse_item(&mut self) -> ParseResult {
        self.parse_fn_decl()
    }

    pub fn parse_fn_decl(&mut self) -> ParseResult {
        let start = self.expect(TokenKind::Fn)?.span.start;
        let ident = self.parse_ident()?;
        let _ = self.expect(TokenKind::LParen)?;
        let _ = self.expect(TokenKind::RParen)?;
        let _ = self.expect(TokenKind::Arrow)?;
        let ty = self.parse_type()?;
        let body = self.parse_block()?;
        let end = self.builder.span_of(body).end;

        Ok(self
            .builder
            .add_fn_decl(Span::new(start, end), ident, Some(ty), body))
    }

    pub fn parse_type(&mut self) -> ParseResult<Ty> {
        let (ty, span) = match self.next()? {
            Some(Token { kind: TokenKind::Never, span }) => (TyKind::Never, span),
            Some(Token { kind: TokenKind::Void, span }) => (TyKind::Void, span),
            Some(Token { kind: TokenKind::UintType(size), span }) => (TyKind::Uint(size), span),
            Some(Token { kind: TokenKind::IntType(size), span }) => (TyKind::Int(size), span),
            Some(Token { kind: TokenKind::FloatType(size), span }) => (TyKind::Float(size), span),
            Some(Token { kind: TokenKind::Identifier, span }) => {
                let (path_id, span) = match self.expect_opt(TokenKind::Identifier)? {
                    Some(Token { kind: TokenKind::DoubleColon, .. }) => {
                        let path_id = self.parse_path(false)?;
                        let path_span = self.builder.span_of(path_id);
                        (path_id, span.join(&path_span))
                    }
                    _ => {
                        let idx = self.intern(&span);
                        let ident = Ident::new(idx, span.clone());
                        let path_id = self.builder.add_path(span.clone(), ident, None);
                        (path_id, span)
                    }
                };
                (TyKind::Other(path_id), span)
            }
            Some(Token { kind, span }) => {
                return Err(ParserError::InvalidType { found: kind, span });
            }
            None => return Err(ParserError::LexerError(LexerError::UnexpectedEof)),
        };
        let ty = Ty { span, kind: ty };
        Ok(ty)
    }

    pub fn parse_path(&mut self, root: bool) -> ParseResult {
        let ident = match self.next()? {
            Some(Token { kind: TokenKind::Identifier | TokenKind::Super, span }) => {
                let idx = self.intern(&span);
                Ident::new(idx, span)
            }
            Some(Token { kind: TokenKind::Package, span }) if root => {
                let idx = self.intern(&span);
                Ident::new(idx, span)
            }
            Some(Token { kind, span }) => {
                return Err(ParserError::InvalidPath { found: kind, span });
            }
            None => return Err(ParserError::LexerError(LexerError::UnexpectedEof)),
        };
        match self.expect_opt(TokenKind::DoubleColon)? {
            Some(_) => {
                let next = self.parse_path(false)?;
                let span = ident.span.join(&self.builder.span_of(next));
                let node = self.builder.add_path(span, ident, Some(next));
                Ok(node)
            }
            None => {
                let node = self.builder.add_path(ident.span.clone(), ident, None);
                Ok(node)
            }
        }
    }

    pub fn parse_block(&mut self) -> ParseResult {
        let start = self.expect(TokenKind::LBrace)?.span.start;
        let end = self.expect(TokenKind::RBrace)?.span.end;

        let span = Span::new(start, end);
        let node = self.builder.add_block(span, None);
        Ok(node)
    }

    pub fn parse_ident(&mut self) -> ParseResult<Ident> {
        let tok = self.expect(TokenKind::Identifier)?;
        let idx = self.intern(&tok.span);
        Ok(Ident::new(idx, tok.span))
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

    fn intern(&mut self, span: &Span) -> InternId {
        let s = span.slice(self.lexer.source());
        let mut session = self.session.lock().unwrap();
        session.interner.get_or_intern(s)
    }
}
