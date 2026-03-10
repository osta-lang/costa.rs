use miette::NamedSource;
use osta_ast::ast::{AstNodeKind, FnDecl, Interned, InternedKind, ItemId};
use osta_ast::NodeId;
use osta_parser::{parse, FileSession};
use osta_session::interner::InternId;
use osta_syntax::Span;

#[test]
fn simplest() {
    let src = include_str!("../../../examples/simplest.osta");

    let FileSession { builder, interner, .. } = parse(src)
        .map_err(|err| {
            err.with_source_code(NamedSource::new("../../../examples/simplest.osta", src))
        })
        .unwrap();
    let ast = builder.build();

    assert_eq!(interner.resolve(InternId::START), "main");
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
    let src = include_str!("../../../examples/simple.osta");

    let FileSession { builder, interner, .. } = parse(src)
        .map_err(|err| err.with_source_code(NamedSource::new("../../../examples/simple.osta", src)))
        .unwrap();
    let ast = builder.build();

    assert_eq!(interner.resolve(InternId::START), "main");
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
