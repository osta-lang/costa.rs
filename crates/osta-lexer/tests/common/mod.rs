macro_rules! init_lexer {
    ($src:ident, $lexer:ident, $slice:literal) => {
        let $src = $slice;
        let mut $lexer = ::osta_lexer::Lexer::new($src);
    };
}

macro_rules! assert_next {
    ($src:expr, $lexer:ident, TOKEN, $kind:pat, $slice:literal) => {
        match $lexer.next() {
            Some(Ok(token)) => match token.kind {
                $kind => assert_eq!(token.span.slice($src), $slice),
                _ => panic!(
                    "expected `{}` ({:?}), got `{}` ({:?})",
                    $slice,
                    stringify!($kind),
                    token.span.slice($src),
                    token.kind
                ),
            },
            Some(Err(e)) => {
                panic!("expected `{}` ({:?}), got error {:?}", $slice, stringify!($kind), e)
            }
            None => panic!("expected `{}` ({:?}), got None", $slice, stringify!($kind)),
        }
    };
    ($src:expr, $lexer:ident, ERROR, $err:pat) => {
        match $lexer.next() {
            Some(Err(e)) => match e {
                $err => {}
                _ => panic!("expected error {:?}, got error {:?}", stringify!($err), e),
            },
            Some(Ok(token)) => panic!(
                "expected error {:?}, got `{}` ({:?})",
                stringify!($err),
                token.span.slice($src),
                token.kind
            ),
            None => panic!("expected error {:?}, got None", stringify!($err)),
        }
    };
    ($src:expr, $lexer:ident, EOF) => {
        match $lexer.next() {
            Some(Ok(token)) => {
                panic!("expected EOF, got `{}` ({:?})", token.span.slice($src), token.kind)
            }
            Some(Err(e)) => panic!("expected EOF, got error {:?}", e),
            None => {}
        }
    };
}
