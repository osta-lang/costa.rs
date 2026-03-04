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
    #[inline]
    pub(crate) fn new(span: Span, kind: AstNodeKind) -> Self {
        Self { span, kind }
    }

    #[inline]
    pub(crate) fn fn_decl(span: Span, ident: Interned, ty: Option<NodeId>, body: NodeId) -> Self {
        Self::new(span, AstNodeKind::fn_decl(ident, ty, body))
    }

    #[inline]
    pub(crate) fn fn_call(span: Span, path_id: NodeId, first_arg_id: Option<NodeId>) -> Self {
        Self::new(span, AstNodeKind::fn_call(path_id, first_arg_id))
    }

    #[inline]
    pub(crate) fn path(span: Span, this: Interned, next: Option<NodeId>) -> Self {
        Self::new(span, AstNodeKind::path(this, next))
    }

    #[inline]
    pub(crate) fn block(span: Span, first_stmt_id: Option<NodeId>) -> Self {
        Self::new(span, AstNodeKind::block(first_stmt_id))
    }

    #[inline]
    pub(crate) fn literal(span: Span, lit: Interned) -> Self {
        Self::new(span, AstNodeKind::literal(lit))
    }

    #[inline]
    pub(crate) fn chain(span: Span, first: NodeId, second: NodeId) -> Self {
        Self::new(span, AstNodeKind::chain(first, second))
    }

    #[inline]
    pub(crate) fn variant_inst(span: Span, path_id: NodeId, first_arg_id: Option<NodeId>) -> Self {
        Self::new(span, AstNodeKind::variant_inst(path_id, first_arg_id))
    }

    #[inline]
    pub(crate) fn if_expr(
        span: Span,
        cond: NodeId,
        then_expr: NodeId,
        else_expr: Option<NodeId>,
    ) -> Self {
        Self::new(span, AstNodeKind::if_expr(cond, then_expr, else_expr))
    }

    #[inline]
    pub(crate) fn ty(span: Span, ty: Ty) -> Self {
        Self::new(span, AstNodeKind::ty(ty))
    }

    #[inline]
    pub(crate) fn variable_binding(
        span: Span,
        ident: Interned,
        ty: Option<NodeId>,
        expr: Option<NodeId>,
    ) -> Self {
        Self::new(span, AstNodeKind::variable_binding(ident, ty, expr))
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
    VariantInst(NodeId, Option<NodeId>),
    IfExpr(NodeId, NodeId, Option<NodeId>),
    Ty(Ty),
    VariableBinding(VariableBinding),
}

impl AstNodeKind {
    #[inline]
    pub fn fn_decl(ident: Interned, ty: Option<NodeId>, body: NodeId) -> Self {
        Self::FnDecl(FnDecl { ident, ty, body })
    }

    #[inline]
    pub fn fn_call(path_id: NodeId, first_arg_id: Option<NodeId>) -> Self {
        Self::FnCall(FnCall { path_id, first_arg_id })
    }

    #[inline]
    pub fn path(this: Interned, next: Option<NodeId>) -> Self {
        Self::Path(Path { this, next })
    }

    #[inline]
    pub fn block(first_stmt_id: Option<NodeId>) -> Self {
        Self::Block(first_stmt_id)
    }

    #[inline]
    pub fn literal(lit: Interned) -> Self {
        Self::Literal(lit)
    }

    #[inline]
    pub fn chain(first: NodeId, second: NodeId) -> Self {
        Self::Chain(first, second)
    }

    #[inline]
    pub fn variant_inst(path_id: NodeId, first_arg_id: Option<NodeId>) -> Self {
        Self::VariantInst(path_id, first_arg_id)
    }

    #[inline]
    pub fn if_expr(cond: NodeId, then_expr: NodeId, else_expr: Option<NodeId>) -> Self {
        Self::IfExpr(cond, then_expr, else_expr)
    }

    #[inline]
    pub fn ty(ty: Ty) -> Self {
        Self::Ty(ty)
    }

    #[inline]
    pub fn variable_binding(ident: Interned, ty: Option<NodeId>, expr: Option<NodeId>) -> Self {
        Self::VariableBinding(VariableBinding { ident, ty, expr })
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct FnDecl {
    pub ident: Interned,
    pub ty: Option<NodeId>,
    pub body: NodeId,
}

#[derive(Debug, Eq, PartialEq)]
pub struct FnCall {
    pub path_id: NodeId,
    pub first_arg_id: Option<NodeId>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Ty {
    Never,
    Void,
    Uint(usize),
    Int(usize),
    Float(usize),
    Path(NodeId),
    Pointer(NodeId),
    Reference(NodeId),
    Array {
        ty: NodeId,
        count: Option<NodeId>,
        terminator: Option<NodeId>,
    },
    Slice(NodeId),
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

#[derive(Debug, Eq, PartialEq)]
pub struct VariableBinding {
    pub ident: Interned,
    pub ty: Option<NodeId>,
    pub expr: Option<NodeId>,
}
