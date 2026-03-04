use crate::ast::{AstNode, Interned, InternedKind, ItemId, NodeId, Ty};
use crate::AST;
use osta_index::{Idx, IndexVec};
use osta_session::interner::InternId;
use osta_syntax::Span;

pub struct AstBuilder {
    nodes: IndexVec<NodeId, AstNode>,
    items: IndexVec<ItemId, NodeId>,
}

impl AstBuilder {
    pub fn new() -> Self {
        Self { nodes: IndexVec::new(), items: IndexVec::new() }
    }

    pub fn build(self) -> AST {
        AST { nodes: self.nodes, items: self.items }
    }

    pub fn checkpoint(&mut self) -> Checkpoint {
        Checkpoint { idx: self.nodes.next_index(), builder: self }
    }

    pub fn span_of(&self, id: NodeId) -> Span {
        self.nodes[id].span.clone()
    }

    pub fn add_item(&mut self, root: NodeId) -> ItemId {
        self.items.push(root)
    }

    fn add_node(&mut self, node: AstNode) -> NodeId {
        self.nodes.push(node)
    }

    pub fn add_fn_decl(
        &mut self,
        span: Span,
        ident: Interned,
        ty: Option<NodeId>,
        body: NodeId,
    ) -> NodeId {
        let node = AstNode::fn_decl(span, ident, ty, body);
        self.add_node(node)
    }

    pub fn add_fn_call(
        &mut self,
        span: Span,
        path_id: NodeId,
        first_arg_id: Option<NodeId>,
    ) -> NodeId {
        let node = AstNode::fn_call(span, path_id, first_arg_id);
        self.add_node(node)
    }

    pub fn add_path(&mut self, span: Span, this: Interned, next: Option<NodeId>) -> NodeId {
        let node = AstNode::path(span, this, next);
        self.add_node(node)
    }

    pub fn add_block(&mut self, span: Span, first_stmt_id: Option<NodeId>) -> NodeId {
        let node = AstNode::block(span, first_stmt_id);
        self.add_node(node)
    }

    pub fn add_literal(&mut self, span: Span, lit: InternId, kind: InternedKind) -> NodeId {
        let node = AstNode::literal(span.clone(), Interned::new(lit, span, kind));
        self.add_node(node)
    }

    pub fn add_chain(&mut self, span: Span, first: NodeId, second: NodeId) -> NodeId {
        let node = AstNode::chain(span, first, second);
        self.add_node(node)
    }

    pub fn add_variant_inst(
        &mut self,
        span: Span,
        path_id: NodeId,
        first_arg_id: Option<NodeId>,
    ) -> NodeId {
        let node = AstNode::variant_inst(span, path_id, first_arg_id);
        self.add_node(node)
    }

    pub fn add_if_expr(
        &mut self,
        span: Span,
        cond: NodeId,
        then_branch: NodeId,
        else_branch: Option<NodeId>,
    ) -> NodeId {
        let node = AstNode::if_expr(span, cond, then_branch, else_branch);
        self.add_node(node)
    }

    pub fn add_type(&mut self, span: Span, ty: Ty) -> NodeId {
        let node = AstNode::ty(span, ty);
        self.add_node(node)
    }

    pub fn add_variable_binding(&mut self, span: Span, ident: Interned, ty: Option<NodeId>, expr: Option<NodeId>) -> NodeId {
        let node = AstNode::variable_binding(span, ident, ty, expr);
        self.add_node(node)
    }
}

impl Default for AstBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Checkpoint {
    idx: NodeId,
    builder: *mut AstBuilder,
}

impl Checkpoint {
    pub fn rollback(self) {
        let builder = unsafe { &mut *self.builder };
        builder.nodes.truncate(self.idx.index());
        std::mem::forget(self);
    }

    pub fn commit(self) {
        std::mem::forget(self);
    }

    pub fn resolve<T, E>(self, result: Result<T, E>) -> Result<T, E> {
        match result {
            Ok(v) => {
                self.commit();
                Ok(v)
            }
            Err(e) => {
                self.rollback();
                Err(e)
            }
        }
    }
}

impl Drop for Checkpoint {
    fn drop(&mut self) {
        panic!("Checkpoint must be either committed or rolled back");
    }
}
