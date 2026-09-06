use crate::expr::Expr;
use crate::rewrite::RewriteEngine;
use crate::space::Law;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProofStep {
    pub law: String,
    pub before: String,
    pub after: String,
}

pub fn replay(start: &Expr, laws: &[Law], proof: &[String]) -> Option<Expr> {
    let named: Vec<&Law> = proof
        .iter()
        .map(|name| laws.iter().find(|law| law.name == *name))
        .collect::<Option<Vec<_>>>()?;

    let mut current = start.clone();
    for law in named {
        let (next, applied) = RewriteEngine::bottom_up(&current, std::slice::from_ref(law), 1);
        if applied.first() != Some(&law.name) {
            return None;
        }
        current = next;
    }
    Some(current)
}

pub fn verify(start: &Expr, expected: &Expr, laws: &[Law], proof: &[String]) -> bool {
    match replay(start, laws, proof) {
        Some(result) => result == *expected,
        None => false,
    }
}
