use crate::ast::{AstNodeKind, ItemId};
use crate::{NodeId, AST};
use osta_syntax::Span;

pub trait AstVisitor {
    fn visit(&mut self, ast: &AST) {
        for (item_id, &node_id) in ast.items.enumerate() {
            self.visit_node(ast, item_id, node_id);
        }
    }

    fn visit_node(&mut self, ast: &AST, item_id: ItemId, node_id: NodeId) {
        let node = &ast.nodes[node_id];
        self.accept(ast, item_id, &node.span, &node.kind);
    }

    fn accept(&mut self, ast: &AST, item_id: ItemId, span: &Span, node: &AstNodeKind);
}
