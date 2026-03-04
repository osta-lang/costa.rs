use crate::tests::assert_path;
use osta_ast::ast::{AstNodeKind, FnCall, Interned, InternedKind, Path};
use osta_ast::{AstBuilder, NodeId};
use osta_lexer::Lexer;
use osta_session::Session;
use osta_syntax::Span;
use std::sync::{Arc, Mutex};

mod unary;

#[test]
fn simple() {
    let session = Arc::new(Mutex::new(Session::new()));
    let mut lexer = Lexer::new("-4 * 7");
    let mut builder = AstBuilder::new();
    let (idx, span) = crate::expr::parse_expr(session.clone(), &mut lexer, &mut builder, 0)
        .expect("failed to parse");
    let ast = builder.build();

    assert_eq!(span, Span::new(0, 6));

    let node = &ast[idx];
    match &node.kind {
        AstNodeKind::FnCall(fn_call) => match fn_call {
            FnCall { path_id, first_arg_id }
                if *path_id == NodeId::from_ty(10)
                    && *first_arg_id == Some(NodeId::from_ty(11)) =>
            {
                assert_eq!(
                    &ast[first_arg_id.expect("first_arg_id is None")].kind,
                    &AstNodeKind::Chain(NodeId::from_ty(5), NodeId::from_ty(6))
                );
                assert_path!(
                    session,
                    ast,
                    *path_id,
                    Span::new(3, 4),
                    core::ops::Multiplication::mul
                );
            }
            _ => panic!("expected fn call with path_id 10 and first_arg_id 11, got {:?}", fn_call),
        },
        _ => panic!("expected fn call, got {:?}", node.kind),
    }
}
