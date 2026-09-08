use std::collections::HashMap;

use crate::expr::{AtomValue, Expr};

pub type Bindings = HashMap<String, Expr>;

pub fn match_expr(pattern: &Expr, target: &Expr, bindings: &Bindings) -> Option<Bindings> {
    match (pattern, target) {
        (Expr::Variable { name, ty }, _) => {
            if let (Some(pt), Some(tt)) = (ty.as_ref(), target.ty()) {
                if pt != tt {
                    return None;
                }
            }
            if let Some(bound) = bindings.get(name) {
                if bound == target {
                    return Some(bindings.clone());
                }
                return None;
            }
            let mut next = bindings.clone();
            next.insert(name.clone(), target.clone());
            Some(next)
        }
        (
            Expr::Atom {
                value: AtomValue::Int(a),
                ..
            },
            Expr::Atom {
                value: AtomValue::Int(b),
                ..
            },
        ) if a == b => Some(bindings.clone()),
        (
            Expr::Atom {
                value: AtomValue::Float(a),
                ..
            },
            Expr::Atom {
                value: AtomValue::Float(b),
                ..
            },
        ) if a == b => Some(bindings.clone()),
        (
            Expr::Atom {
                value: AtomValue::Str(a),
                ..
            },
            Expr::Atom {
                value: AtomValue::Str(b),
                ..
            },
        ) if a == b => Some(bindings.clone()),
        (Expr::Symbol { name: s, .. }, Expr::Symbol { name: t, .. }) if s == t => {
            Some(bindings.clone())
        }
        (
            Expr::Apply {
                head: ph, args: pa, ..
            },
            Expr::Apply {
                head: th, args: ta, ..
            },
        ) => {
            if pa.len() != ta.len() {
                return None;
            }
            let mut current = match_expr(ph, th, bindings)?;
            for (p_arg, t_arg) in pa.iter().zip(ta.iter()) {
                current = match_expr(p_arg, t_arg, &current)?;
            }
            Some(current)
        }
        _ => None,
    }
}

pub fn substitute(expr: &Expr, bindings: &Bindings) -> Expr {
    match expr {
        Expr::Variable { name, .. } => bindings.get(name).cloned().unwrap_or_else(|| expr.clone()),
        Expr::Apply { head, args, .. } => Expr::apply(
            substitute(head, bindings),
            args.iter().map(|arg| substitute(arg, bindings)).collect(),
        ),
        _ => expr.clone(),
    }
}
