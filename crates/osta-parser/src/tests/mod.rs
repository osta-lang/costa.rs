mod expr;

macro_rules! assert_path {
    ($interner:expr, $ast:expr, $idx:expr, $span:expr, $path:ident) => {
        let interned = $interner.get_or_intern(stringify!($path));
        let interned = Interned {
            idx: interned,
            span: $span.clone(),
            kind: InternedKind::Ident,
        };
        let node = &$ast[$idx];
        match &node.kind {
            AstNodeKind::Path(p) => {
                assert_eq!(p.this, interned);
                assert!(p.next.is_none());
            },
            _ => panic!("expected path, got {:?}", node.kind),
        }
    };
    ($interner:expr, $ast:expr, $idx:expr, $span:expr, $path:ident :: $($rest:ident)::+) => {
        let interned = $interner.get_or_intern(stringify!($path));
        let interned = Interned {
            idx: interned,
            span: $span.clone(),
            kind: InternedKind::Ident,
        };
        let path = Path {
            this: interned,
            next: Some($idx - 1u32),
        };
        let node = &$ast[$idx];
        match &node.kind {
            AstNodeKind::Path(p) => {assert_eq!(p, &path);},
            _ => panic!("expected path, got {:?}", node.kind),
        }
        assert_path!($interner, $ast, $idx - 1u32, $span, $($rest)::+);
    };
}

pub(crate) use assert_path;
