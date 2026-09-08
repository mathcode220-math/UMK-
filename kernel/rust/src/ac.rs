use crate::expr::Expr;
use crate::space::{OpProperty, Space};

pub fn canonicalize(expr: &Expr, space: &Space) -> Expr {
    match expr {
        Expr::Apply { head, args, .. } => {
            let head = canonicalize(head, space);
            let args: Vec<Expr> = args.iter().map(|arg| canonicalize(arg, space)).collect();
            let mut current = Expr::apply(head.clone(), args);
            if let Some(op) = Space::op_name(&head) {
                if space.declares(op, OpProperty::Associative) {
                    current = flatten_op(&current, op);
                }
                if space.declares(op, OpProperty::Commutative) {
                    current = sort_op(&current, op);
                }
            }
            current
        }
        _ => expr.clone(),
    }
}

pub fn ac_eq(left: &Expr, right: &Expr, space: &Space) -> bool {
    canonicalize(left, space) == canonicalize(right, space)
}

fn flatten_op(expr: &Expr, op: &str) -> Expr {
    match expr {
        Expr::Apply { head, args, .. } if Space::op_name(head) == Some(op) => {
            let mut flat = Vec::new();
            for arg in args {
                let arg = flatten_op(arg, op);
                match arg {
                    Expr::Apply {
                        head: inner_head,
                        args: inner_args,
                        ..
                    } if Space::op_name(&inner_head) == Some(op) => {
                        flat.extend(inner_args);
                    }
                    other => flat.push(other),
                }
            }
            Expr::apply(head.as_ref().clone(), flat)
        }
        _ => expr.clone(),
    }
}

fn sort_op(expr: &Expr, op: &str) -> Expr {
    match expr {
        Expr::Apply { head, args, .. } if Space::op_name(head) == Some(op) => {
            let mut args = args.clone();
            args.sort_by_key(|arg| format!("{arg}"));
            Expr::apply(head.as_ref().clone(), args)
        }
        _ => expr.clone(),
    }
}
