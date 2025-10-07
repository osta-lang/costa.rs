use osta_ast::ast::{AstNodeKind, FnDecl, Interned, InternedKind, ItemId, Ty, TyKind};
use osta_ast::NodeId;
use osta_parser::parse;
use osta_session::interner::InternId;
use osta_session::Session;
use osta_syntax::Span;
use std::sync::{Arc, Mutex};

#[test]
fn simple() {
    let session = Arc::new(Mutex::new(Session::new()));
    let src = include_str!("../../examples/simple.osta");

    let ast = parse(session.clone(), src).expect("parse error");

    {
        let session = session.lock().unwrap();
        assert_eq!(session.interner.resolve(InternId::START), "main");
    }
    assert_eq!(ast[ItemId::START], NodeId::from_ty(1));
    let fn_node = &ast[NodeId::from_ty(1)];
    assert_eq!(fn_node.span, Span::new(0, 20));
    assert_eq!(
        fn_node.kind,
        AstNodeKind::FnDecl(FnDecl {
            ident: Interned {
                idx: InternId::START,
                span: Span::new(3, 7),
                kind: InternedKind::Ident
            },
            ty: Some(Ty { kind: TyKind::Int(32), span: Span::new(13, 16) }),
            body: NodeId::START,
        })
    );
    let body_node = &ast[NodeId::START];
    assert_eq!(body_node.span, Span::new(17, 20));
    assert_eq!(body_node.kind, AstNodeKind::Block(None));
}
