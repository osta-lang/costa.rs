use osta_ast::ast::{AstNodeKind, FnDecl, Interned, InternedKind, ItemId};
use osta_ast::NodeId;
use osta_parser::parse;
use osta_session::interner::InternId;
use osta_session::Session;
use osta_syntax::Span;

#[test]
fn simplest() {
    let mut session = Session::create();
    let src = include_str!("../../../examples/simplest.osta");

    let ast = parse(&mut session, src).expect("parse error");

    assert_eq!(session.resolve_intern(InternId::START), "main");
    assert_eq!(ast[ItemId::START], NodeId::from_ty(2));
    let fn_node = &ast[NodeId::from_ty(2)];
    assert_eq!(fn_node.span, Span::new(0, 20));
    assert_eq!(
        fn_node.kind,
        AstNodeKind::FnDecl(FnDecl {
            ident: Interned {
                idx: InternId::START,
                span: Span::new(3, 7),
                kind: InternedKind::Ident
            },
            args: None,
            ty: Some(NodeId::START),
            body: NodeId::START + 1,
        })
    );
    let body_node = &ast[NodeId::START + 1];
    assert_eq!(body_node.span, Span::new(17, 20));
    assert_eq!(body_node.kind, AstNodeKind::Block(None));
}

#[test]
fn simple() {
    let mut session = Session::create();
    let src = include_str!("../../../examples/simple.osta");

    let ast = parse(&mut session, src).expect("parse error");

    assert_eq!(session.resolve_intern(InternId::START), "main");
    assert_eq!(ast[ItemId::START], NodeId::from_ty(26));
    let fn_node = &ast[NodeId::from_ty(26)];
    assert_eq!(fn_node.span, Span::new(0, 66));
    assert_eq!(
        fn_node.kind,
        AstNodeKind::FnDecl(FnDecl {
            ident: Interned {
                idx: InternId::START,
                span: Span::new(3, 7),
                kind: InternedKind::Ident
            },
            args: None,
            ty: Some(NodeId::START),
            body: NodeId::START + 25,
        })
    );
    let body_node = &ast[NodeId::START + 25];
    assert_eq!(body_node.span, Span::new(17, 66));
    assert_eq!(body_node.kind, AstNodeKind::Block(Some(NodeId::START + 24)));
}
