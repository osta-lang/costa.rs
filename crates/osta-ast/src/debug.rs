use crate::ast::{AstNodeKind, FnCall, FnDecl, Interned, InternedKind, Path, Ty, VariableBinding};
use crate::{NodeId, AST};
use osta_index::Idx;
use osta_session::interner::{InternId, Interner};
use std::alloc::Allocator;
use std::fmt::{Display, Formatter, Result as FmtResult};

pub struct AstPrinter<'src, 'ast, A: Allocator> {
    interner: &'ast Interner<A>,
    source: &'src str,
    ast: &'ast AST,
}

impl<'src, 'ast, A: Allocator> AstPrinter<'src, 'ast, A> {
    pub fn new(interner: &'ast Interner<A>, source: &'src str, ast: &'ast AST) -> Self {
        Self { interner, source, ast }
    }

    fn resolve(&self, id: InternId) -> &'ast str {
        self.interner.resolve(id)
    }

    fn slice(&self, node_id: NodeId) -> &'src str {
        self.ast.nodes[node_id].span.slice(self.source)
    }

    fn print_dot(&self, f: &mut Formatter<'_>, node_id: NodeId) -> FmtResult {
        let node = &self.ast.nodes[node_id];
        let name = match &node.kind {
            AstNodeKind::FnDecl(FnDecl { ident, .. }) => {
                let name = self.resolve(ident.idx);
                format!("FnDecl : {name}")
            }
            AstNodeKind::FnCall(FnCall { path_id, first_arg_id }) => {
                let path = get_path(self.interner, self.ast, *path_id);
                first_arg_id
                    .map(|arg| get_call_args(self.ast, arg, self.source))
                    .map(|args| format!("{path}({args})"))
                    .map(|text| {
                        if text.contains("\n") {
                            "<multiline>".to_string()
                        } else {
                            text
                        }
                    })
                    .map(|text| format!("FnCall : {text}"))
                    .unwrap_or_else(|| format!("FnCall : {path}"))
            }
            AstNodeKind::Path(Path { this, .. }) => {
                let name = self.resolve(this.idx);
                format!("Path : {name}")
            }
            AstNodeKind::Block(_) => "Block".to_string(),
            AstNodeKind::Literal(Interned { idx, kind, .. }) => {
                let ty = match kind {
                    InternedKind::Ident => "Ident",
                    InternedKind::DecInt => "DecInt",
                    InternedKind::BinInt => "BinInt",
                    InternedKind::OctInt => "OctInt",
                    InternedKind::HexInt => "HexInt",
                    InternedKind::Float => "Float",
                    InternedKind::IntFloat => "IntFloat",
                    InternedKind::FloatExp => "FloatExp",
                    InternedKind::IntExp => "IntExp",
                    InternedKind::Str => "Str",
                    InternedKind::RawStr => "RawStr",
                    InternedKind::Char => "Char",
                };
                let content = self.resolve(*idx);
                format!("Literal<{ty}> : {content}")
            }
            AstNodeKind::Chain(_, _) => "Chain".to_string(),
            AstNodeKind::VariantInst(_, _) => "Variant Instantiation".to_string(),
            AstNodeKind::IfExpr(_, _, _) => "If Expression".to_string(),
            AstNodeKind::Ty(ty) => {
                let ty = get_type(self.interner, self.ast, ty);
                format!("Type : {ty}")
            }
            AstNodeKind::VariableBinding(VariableBinding { ident, ty, expr }) => {
                let ident = self.resolve(ident.idx);
                let ty = ty.map(|ty| get_type_by_node(self.interner, self.ast, ty));
                let expr = expr.map(|expr| self.slice(expr));
                ty.map(|ty| {
                    expr.map(|expr| format!("Variable Binding : let {ident}: {ty} = {expr}"))
                        .unwrap_or_else(|| format!("Variable Binding : let {ident}: {ty};"))
                })
                .unwrap_or_else(|| {
                    expr.map(|expr| format!("Variable Binding : let {ident} = {expr}"))
                        .unwrap_or_else(|| format!("Variable Binding : let {ident}"))
                })
            }
        };

        writeln!(
            f,
            "{} [label=\"[{}] {}\"]",
            node_id.index(),
            node.span,
            name.replace('"', r#"\""#)
        )
    }
}

impl<'src, 'ast, A: Allocator> Display for AstPrinter<'src, 'ast, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        writeln!(f, "digraph G {{")?;
        for node_id in self.ast.items.iter() {
            print_dot(self, f, *node_id)?;
        }
        writeln!(f, "}}")
    }
}

fn print_dot<'src, 'ast, A: Allocator>(
    printer: &AstPrinter<'src, 'ast, A>,
    f: &mut Formatter<'_>,
    node_id: NodeId,
) -> FmtResult {
    printer.print_dot(f, node_id)?;

    match &printer.ast.nodes[node_id].kind {
        AstNodeKind::FnDecl(FnDecl { ty, body, .. }) => {
            if let Some(ty) = ty {
                writeln!(f, "{} -> {}", node_id.index(), ty.index())?;
                print_dot(printer, f, *ty)?;
            };
            writeln!(f, "{} -> {}", node_id.index(), body.index())?;
            print_dot(printer, f, *body)?;
        }
        AstNodeKind::FnCall(FnCall { path_id, first_arg_id }) => {
            writeln!(f, "{} -> {}", node_id.index(), path_id.index())?;
            print_dot(printer, f, *path_id)?;
            if let Some(first_arg_id) = first_arg_id {
                writeln!(f, "{} -> {}", node_id.index(), first_arg_id.index())?;
                print_dot(printer, f, *first_arg_id)?;
            }
        }
        AstNodeKind::Path(Path { next, .. }) => {
            if let Some(next) = next {
                writeln!(f, "{} -> {}", node_id.index(), next.index())?;
                print_dot(printer, f, *next)?;
            }
        }
        AstNodeKind::Block(stmts) => {
            if let Some(stmts) = stmts {
                writeln!(f, "{} -> {}", node_id.index(), stmts.index())?;
                print_dot(printer, f, *stmts)?;
            }
        }
        AstNodeKind::Literal(_) => {}
        AstNodeKind::Chain(first, next) => {
            writeln!(f, "{} -> {}", node_id.index(), first.index())?;
            print_dot(printer, f, *first)?;
            writeln!(f, "{} -> {}", node_id.index(), next.index())?;
            print_dot(printer, f, *next)?;
        }
        AstNodeKind::VariantInst(path, args) => {
            writeln!(f, "{} -> {}", node_id.index(), path.index())?;
            print_dot(printer, f, *path)?;
            if let Some(args) = args {
                writeln!(f, "{} -> {}", node_id.index(), args.index())?;
                print_dot(printer, f, *args)?;
            }
        }
        AstNodeKind::IfExpr(cond_expr, then_expr, else_expr) => {
            writeln!(f, "{} -> {}", node_id.index(), cond_expr.index())?;
            print_dot(printer, f, *cond_expr)?;
            writeln!(f, "{} -> {}", node_id.index(), then_expr.index())?;
            print_dot(printer, f, *then_expr)?;
            if let Some(else_expr) = else_expr {
                writeln!(f, "{} -> {}", node_id.index(), else_expr.index())?;
                print_dot(printer, f, *else_expr)?;
            }
        }
        AstNodeKind::Ty(ty) => match ty {
            Ty::Path(ty) => {
                writeln!(f, "{} -> {}", node_id.index(), ty.index())?;
                print_dot(printer, f, *ty)?;
            }
            Ty::Pointer(ty) => {
                writeln!(f, "{} -> {}", node_id.index(), ty.index())?;
                print_dot(printer, f, *ty)?;
            }
            Ty::Reference(ty) => {
                writeln!(f, "{} -> {}", node_id.index(), ty.index())?;
                print_dot(printer, f, *ty)?;
            }
            Ty::Array { ty, count, terminator } => {
                writeln!(f, "{} -> {}", node_id.index(), ty.index())?;
                print_dot(printer, f, *ty)?;
                if let Some(count) = count {
                    writeln!(f, "{} -> {}", node_id.index(), count.index())?;
                    print_dot(printer, f, *count)?;
                }
                if let Some(terminator) = terminator {
                    writeln!(f, "{} -> {}", node_id.index(), terminator.index())?;
                    print_dot(printer, f, *terminator)?;
                }
            }
            Ty::Slice(ty) => {
                writeln!(f, "{} -> {}", node_id.index(), ty.index())?;
                print_dot(printer, f, *ty)?;
            }
            Ty::Other(ty) => {
                writeln!(f, "{} -> {}", node_id.index(), ty.index())?;
                print_dot(printer, f, *ty)?;
            }
            _ => {}
        },
        AstNodeKind::VariableBinding(VariableBinding { ty, expr, .. }) => {
            if let Some(ty) = ty {
                writeln!(f, "{} -> {}", node_id.index(), ty.index())?;
                print_dot(printer, f, *ty)?;
            }
            if let Some(expr) = expr {
                writeln!(f, "{} -> {}", node_id.index(), expr.index())?;
                print_dot(printer, f, *expr)?;
            }
        }
    }

    Ok(())
}

fn get_path<A: Allocator>(interner: &Interner<A>, ast: &AST, path_id: NodeId) -> String {
    let mut name = String::new();
    let mut path_id = Some(path_id);
    while let Some(path) = path_id {
        let this = match &ast.nodes[path].kind {
            AstNodeKind::Path(path) => {
                path_id = path.next;
                path.this.idx
            }
            _ => unreachable!(),
        };
        let this = interner.resolve(this);
        if name.is_empty() {
            name = this.to_string();
        } else {
            name += "::";
            name += this;
        }
    }
    name
}

fn get_call_args(ast: &AST, node: NodeId, source: &str) -> String {
    let node = &ast.nodes[node];
    match node.kind {
        AstNodeKind::Chain(first, next) => {
            let first = get_call_args(ast, first, source);
            let next = get_call_args(ast, next, source);
            format!("{first}, {next}")
        }
        _ => node.span.slice(source).to_string(),
    }
}

fn get_type<A: Allocator>(interner: &Interner<A>, ast: &AST, ty: &Ty) -> String {
    match ty {
        Ty::Never => "Never".to_string(),
        Ty::Void => "Void".to_string(),
        Ty::Uint(bits) => format!("u{bits}"),
        Ty::Int(bits) => format!("i{bits}"),
        Ty::Float(bits) => format!("f{bits}"),
        Ty::Path(node) => get_path(interner, ast, *node),
        Ty::Pointer(ty_id) => {
            let ty = get_type_by_node(interner, ast, *ty_id);
            format!("*{ty}")
        }
        Ty::Reference(ty_id) => {
            let ty = get_type_by_node(interner, ast, *ty_id);
            format!("&{ty}")
        }
        Ty::Array { ty, count, terminator } => {
            let ty = get_type_by_node(interner, ast, *ty);
            format!("[{ty}; {count:?}: {terminator:?}]")
        }
        Ty::Slice(_) => "Slice".to_string(),
        Ty::Other(_) => "Other".to_string(),
    }
}

fn get_type_by_node<A: Allocator>(interner: &Interner<A>, ast: &AST, ty: NodeId) -> String {
    let ty = match &ast.nodes[ty].kind {
        AstNodeKind::Ty(ty) => ty,
        _ => unreachable!(),
    };
    get_type(interner, ast, ty)
}
