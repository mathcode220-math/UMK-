use std::collections::HashMap;

use crate::ac::canonicalize;
use crate::expr::Expr;
use crate::rewrite::RewriteEngine;
use crate::space::Space;

#[derive(Clone, Debug, PartialEq)]
pub enum SemanticValue {
    Opaque(Expr),
}

pub type Environment = HashMap<String, Expr>;

pub fn interpret(expr: &Expr, space: &Space, rho: &Environment, max_steps: usize) -> SemanticValue {
    let substituted = subst_env(expr, rho);
    let (reduced, _) = RewriteEngine::bottom_up(&substituted, &space.laws, max_steps);
    SemanticValue::Opaque(canonicalize(&reduced, space))
}

fn subst_env(expr: &Expr, rho: &Environment) -> Expr {
    match expr {
        Expr::Symbol { name, .. } | Expr::Variable { name, .. } => {
            rho.get(name).cloned().unwrap_or_else(|| expr.clone())
        }
        Expr::Apply { head, args, .. } => Expr::apply(
            subst_env(head, rho),
            args.iter().map(|arg| subst_env(arg, rho)).collect(),
        ),
        other => other.clone(),
    }
}
