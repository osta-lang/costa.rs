use osta_index::{new_index, IndexVec};
use osta_session::interner::InternId;
use osta_syntax::Span;
use std::ops::Index;

new_index!(pub NodeId: u32);
new_index!(pub ItemId: u32);

#[derive(Debug)]
pub struct AST {
    pub(crate) nodes: IndexVec<NodeId, AstNode>,
    pub(crate) items: IndexVec<ItemId, NodeId>,
}

impl AST {
    pub fn get_node(&self, id: NodeId) -> &AstNode {
        &self.nodes[id]
    }
}

impl Index<NodeId> for AST {
    type Output = AstNode;

    fn index(&self, index: NodeId) -> &Self::Output {
        &self.nodes[index]
    }
}

impl Index<ItemId> for AST {
    type Output = NodeId;

    fn index(&self, index: ItemId) -> &Self::Output {
        &self.items[index]
    }
}

#[derive(Debug)]
pub struct AstNode {
    pub span: Span,
    pub kind: AstNodeKind,
}

impl AstNode {
    pub(crate) fn new(span: Span, kind: AstNodeKind) -> Self {
        Self { span, kind }
    }

    pub(crate) fn fn_decl(span: Span, ident: Interned, ty: Option<Ty>, body: NodeId) -> Self {
        Self::new(span, AstNodeKind::fn_decl(ident, ty, body))
    }

    pub(crate) fn fn_call(span: Span, path_id: NodeId, first_arg_id: NodeId) -> Self {
        Self::new(span, AstNodeKind::fn_call(path_id, first_arg_id))
    }

    pub(crate) fn path(span: Span, this: Interned, next: Option<NodeId>) -> Self {
        Self::new(span, AstNodeKind::path(this, next))
    }

    pub(crate) fn block(span: Span, first_stmt_id: Option<NodeId>) -> Self {
        Self::new(span, AstNodeKind::block(first_stmt_id))
    }

    pub(crate) fn literal(span: Span, lit: Interned) -> Self {
        Self::new(span, AstNodeKind::literal(lit))
    }

    pub(crate) fn chain(span: Span, first: NodeId, second: NodeId) -> Self {
        Self::new(span, AstNodeKind::chain(first, second))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum AstNodeKind {
    FnDecl(FnDecl),
    FnCall(FnCall),
    Path(Path),
    Block(Option<NodeId>),
    Literal(Interned),
    Chain(NodeId, NodeId),
}

impl AstNodeKind {
    pub fn fn_decl(ident: Interned, ty: Option<Ty>, body: NodeId) -> Self {
        Self::FnDecl(FnDecl { ident, ty, body })
    }

    pub fn fn_call(path_id: NodeId, first_arg_id: NodeId) -> Self {
        Self::FnCall(FnCall { path_id, first_arg_id })
    }

    pub fn path(this: Interned, next: Option<NodeId>) -> Self {
        Self::Path(Path { this, next })
    }

    pub fn block(first_stmt_id: Option<NodeId>) -> Self {
        Self::Block(first_stmt_id)
    }

    pub fn literal(lit: Interned) -> Self {
        Self::Literal(lit)
    }

    pub fn chain(first: NodeId, second: NodeId) -> Self {
        Self::Chain(first, second)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct FnDecl {
    pub ident: Interned,
    pub ty: Option<Ty>,
    pub body: NodeId,
}

#[derive(Debug, Eq, PartialEq)]
pub struct FnCall {
    pub path_id: NodeId,
    pub first_arg_id: NodeId,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Ty {
    pub span: Span,
    pub kind: TyKind,
}

#[derive(Debug, Eq, PartialEq)]
pub enum TyKind {
    Never,
    Void,
    Uint(usize),
    Int(usize),
    Float(usize),
    Other(NodeId),
}

#[derive(Debug, Eq, PartialEq)]
pub struct Path {
    pub this: Interned,
    pub next: Option<NodeId>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Interned {
    pub idx: InternId,
    pub span: Span,
    pub kind: InternedKind,
}

impl Interned {
    pub fn new(idx: InternId, span: Span, kind: InternedKind) -> Self {
        Self { idx, span, kind }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum InternedKind {
    Ident,
    DecInt,
    BinInt,
    OctInt,
    HexInt,
    Float,
    IntFloat,
    FloatExp,
    IntExp,
    Str,
    RawStr,
    Char,
}
