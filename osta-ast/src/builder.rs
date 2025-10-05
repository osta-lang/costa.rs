use crate::ast::{AstNode, Ident, ItemId, NodeId, Ty};
use crate::AST;
use osta_index::{Idx, IndexVec};
use osta_syntax::Span;

pub struct AstBuilder {
    nodes: IndexVec<NodeId, AstNode>,
    items: IndexVec<ItemId, NodeId>,
}

impl AstBuilder {
    pub fn new() -> Self {
        Self {
            nodes: IndexVec::new(),
            items: IndexVec::new(),
        }
    }

    pub fn build(self) -> AST {
        AST {
            nodes: self.nodes,
            items: self.items,
        }
    }

    pub fn checkpoint(&mut self) -> Checkpoint {
        Checkpoint {
            idx: self.nodes.next_index(),
            builder: self,
        }
    }

    pub fn span_of(&self, id: NodeId) -> Span {
        (&self.nodes[id]).span.clone()
    }

    pub fn add_item(&mut self, root: NodeId) -> ItemId {
        self.items.push(root)
    }

    fn add_node(&mut self, node: AstNode) -> NodeId {
        self.nodes.push(node)
    }

    pub fn add_fn_decl(&mut self, span: Span, ident: Ident, ty: Option<Ty>, body: NodeId) -> NodeId {
        let node = AstNode::fn_decl(span, ident, ty, body);
        self.add_node(node)
    }

    pub fn add_block(&mut self, span: Span, first_stmt_id: Option<NodeId>) -> NodeId {
        let node = AstNode::block(span, first_stmt_id);
        self.add_node(node)
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
