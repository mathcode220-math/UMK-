use std::collections::HashMap;

use crate::expr::Expr;
use crate::pattern::Bindings;
use crate::rewrite::RewriteEngine;
use crate::space::Law;

#[derive(Clone, Debug, PartialEq)]
pub struct Instantiation {
    pub bindings: Vec<(String, Expr)>,
}

impl Instantiation {
    pub fn from_bindings(bindings: &Bindings) -> Self {
        let mut items: Vec<(String, Expr)> = bindings
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        items.sort_by(|a, b| a.0.cmp(&b.0));
        Instantiation { bindings: items }
    }

    pub fn get(&self, name: &str) -> Option<&Expr> {
        self.bindings
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, expr)| expr)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProofObject {
    pub law_id: String,
    pub instantiation: Instantiation,
    pub premises: Vec<ProofObject>,
    pub before: Expr,
    pub after: Expr,
}

impl ProofObject {
    pub fn law(
        law_id: impl Into<String>,
        bindings: &Bindings,
        before: Expr,
        after: Expr,
    ) -> Self {
        ProofObject {
            law_id: law_id.into(),
            instantiation: Instantiation::from_bindings(bindings),
            premises: Vec::new(),
            before,
            after,
        }
    }

    pub fn with_premises(mut self, premises: Vec<ProofObject>) -> Self {
        self.premises = premises;
        self
    }

    pub fn law_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        collect_law_names(self, &mut names);
        names
    }
}

fn collect_law_names(step: &ProofObject, names: &mut Vec<String>) {
    for premise in &step.premises {
        collect_law_names(premise, names);
    }
    names.push(step.law_id.clone());
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Strategy {
    BottomUp,
    Innermost,
    Outermost,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Reduction {
    pub result: Expr,
    pub steps: Vec<ProofObject>,
    pub cycled: bool,
    pub strategy: Strategy,
}

impl Reduction {
    pub fn law_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        for step in &self.steps {
            names.extend(step.law_names());
        }
        names
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProofStep {
    pub law: String,
    pub before: String,
    pub after: String,
}

pub fn replay(start: &Expr, laws: &[Law], proof: &[String]) -> Option<Expr> {
    let mut current = start.clone();
    for name in proof {
        let law = laws.iter().find(|law| law.name == *name)?;
        let (next, applied) = RewriteEngine::innermost(&current, std::slice::from_ref(law), 1);
        if applied.first() != Some(name) {
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

pub fn verify_objects(
    start: &Expr,
    expected: &Expr,
    laws: &[Law],
    proof: &[ProofObject],
) -> bool {
    let mut current = start.clone();
    for step in proof {
        if step.before != current {
            return false;
        }
        let law = match laws.iter().find(|law| law.name == step.law_id) {
            Some(law) => law,
            None => return false,
        };
        let bindings: Bindings = step
            .instantiation
            .bindings
            .iter()
            .cloned()
            .collect::<HashMap<_, _>>();
        let matched = crate::pattern::match_expr(&law.pattern, &current, &Default::default());
        let Some(found) = matched else {
            return false;
        };
        if Instantiation::from_bindings(&found) != step.instantiation {
            return false;
        }
        let after = crate::pattern::substitute(&law.replacement, &bindings);
        if after != step.after {
            return false;
        }
        current = step.after.clone();
    }
    current == *expected
}
