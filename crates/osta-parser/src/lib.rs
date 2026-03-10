mod error;
mod expr;
mod item;
mod path;
mod stmt;
#[cfg(test)]
mod tests;
mod util;

use crate::error::ParseResult;
use crate::item::parse_item;
use crate::util::peek;
use miette::Report;
use osta_alloc::BumpAllocator;
use osta_ast::AstBuilder;
use osta_lexer::Lexer;
use osta_session::interner::{InternId, Interner};
use osta_syntax::Span;
use std::cell::RefCell;

thread_local! {
    static FILE_SESSION: RefCell<Option<FileSession<'static>>> = RefCell::new(None);
}

pub struct FileSession<'src> {
    pub lexer: Lexer<'src>,
    pub interner: Interner<BumpAllocator>,
    pub builder: AstBuilder,
}

impl<'src> FileSession<'src> {
    pub(crate) fn start(source: &'src str) {
        let source: &'static str =
            unsafe { std::mem::transmute::<&'src str, &'static str>(source) };

        let lexer = Lexer::new(source);
        let interner = Interner::new_in(BumpAllocator::new());
        let builder = AstBuilder::new();
        let session = FileSession { lexer, interner, builder };

        FILE_SESSION.with_borrow_mut(|option| *option = Some(session));
    }
}

impl FileSession<'static> {
    #[inline(always)]
    fn end<'src>() -> FileSession<'src> {
        let session =
            FILE_SESSION.with_borrow_mut(|option| unsafe { option.take().unwrap_unchecked() });
        unsafe { std::mem::transmute(session) }
    }

    #[inline(always)]
    pub(crate) fn get() -> &'static mut FileSession<'static> {
        FILE_SESSION.with_borrow_mut(|session| unsafe { std::mem::transmute(session) })
    }

    #[inline(always)]
    pub(crate) fn lexer() -> &'static mut Lexer<'static> {
        &mut Self::get().lexer
    }

    #[inline(always)]
    pub(crate) fn intern(span: &Span) -> InternId {
        let lexer = Self::lexer();
        let s = span.slice(lexer.source());
        Self::intern_str(s)
    }

    #[inline(always)]
    pub(crate) fn intern_str(s: &str) -> InternId {
        Self::get().interner.get_or_intern(s)
    }

    #[inline(always)]
    pub(crate) fn builder() -> &'static mut AstBuilder {
        &mut Self::get().builder
    }
}

pub fn parse(source: &'_ str) -> Result<FileSession<'_>, Report> {
    FileSession::start(source);
    parse_root().map_err(|err| {
        FileSession::end();
        err.0
    })?;
    let session = FileSession::end();
    Ok(session)
}

pub fn parse_root() -> ParseResult<()> {
    let builder = FileSession::builder();

    while peek()?.is_some() {
        let id = try_parse!(parse_item)?;
        builder.add_item(id);
    }
    Ok(())
}
