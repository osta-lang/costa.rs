mod error;
mod expr;
mod item;
mod path;
mod stmt;
#[cfg(test)]
mod tests;
mod util;

pub use crate::error::{ParseResult, ParseResultOpt, ParserError};
use crate::item::parse_item;
use crate::util::peek;
use osta_ast::{AstBuilder, AST};
use osta_lexer::Lexer;
use osta_session::Session;

pub fn parse(session: &mut Session, source: &str) -> ParseResult<AST> {
    let mut lexer = Lexer::new(source);
    let mut builder = AstBuilder::new();
    parse_root(session, &mut lexer, &mut builder)?;
    Ok(builder.build())
}

pub fn parse_root<'src>(
    session: &mut Session,
    lexer: &mut Lexer<'src>,
    builder: &mut AstBuilder,
) -> ParseResult<()> {
    while peek(lexer)?.is_some() {
        let id = builder
            .checkpoint()
            .resolve(parse_item(session, lexer, builder))?;
        builder.add_item(id);
    }
    Ok(())
}
