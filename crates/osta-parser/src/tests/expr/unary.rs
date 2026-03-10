mod unary {
    use crate::tests::assert_path;
    use crate::FileSession;
    use osta_ast::ast::{AstNodeKind, FnCall, Interned, InternedKind, Path};
    use osta_ast::{AstBuilder, NodeId};
    use osta_lexer::Lexer;
    use osta_session::interner::InternId;
    use osta_session::Session;
    use osta_syntax::Span;

    #[test]
    fn minus() {
        FileSession::start("-1");
        let (idx, span) = crate::expr::parse_expr(0).expect("failed to parse");
        let FileSession { builder, mut interner, .. } = FileSession::end();
        let ast = builder.build();

        assert_eq!(span, Span::new(0, 2));

        let node = &ast[idx];
        match &node.kind {
            AstNodeKind::FnCall(fn_call) => match fn_call {
                FnCall { path_id, first_arg_id }
                    if *path_id == NodeId::from_ty(4) && *first_arg_id == Some(NodeId::START) =>
                {
                    assert_eq!(
                        &ast[first_arg_id.expect("first_arg_id is None")].kind,
                        &AstNodeKind::Literal(Interned::new(
                            InternId::from_ty(0),
                            Span::new(1, 2),
                            InternedKind::DecInt
                        ))
                    );
                    assert_path!(interner, ast, *path_id, Span::new(0, 1), core::ops::Neg::neg);
                }
                _ => panic!(
                    "expected fn call with path_id 4 and first_arg_id START, got {:?}",
                    fn_call
                ),
            },
            _ => panic!("expected fn call, got {:?}", node.kind),
        }
    }
}
