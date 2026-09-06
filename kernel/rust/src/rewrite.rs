use std::collections::HashSet;

use crate::expr::Expr;
use crate::intern::{Interner, NodeId};
use crate::pattern::{match_expr, substitute};
use crate::space::Law;

pub struct RewriteEngine;

impl RewriteEngine {
    pub fn apply_once(expr: &Expr, laws: &[Law]) -> Option<(Expr, String)> {
        for law in laws {
            if let Some(bindings) = match_expr(&law.pattern, expr, &Default::default()) {
                let result = substitute(&law.replacement, &bindings);
                return Some((result, law.name.clone()));
            }
        }
        None
    }

    pub fn bottom_up(expr: &Expr, laws: &[Law], max_steps: usize) -> (Expr, Vec<String>) {
        let mut interner = Interner::new();
        Self::bottom_up_with(&mut interner, expr, laws, max_steps)
    }

    fn bottom_up_with(
        interner: &mut Interner,
        expr: &Expr,
        laws: &[Law],
        max_steps: usize,
    ) -> (Expr, Vec<String>) {
        let mut proof = Vec::new();
        let mut current = expr.clone();
        let mut seen: HashSet<NodeId> = HashSet::new();

        for _ in 0..max_steps {
            if !seen.insert(interner.intern(&current)) {
                break;
            }

            let mut changed = false;
            if let Expr::Apply { head, args, .. } = &current {
                let mut new_args = Vec::with_capacity(args.len());
                for arg in args {
                    let (reduced, arg_proof) = Self::bottom_up_with(interner, arg, laws, 1);
                    if !arg_proof.is_empty() {
                        proof.extend(arg_proof);
                        changed = true;
                    }
                    new_args.push(reduced);
                }
                if changed {
                    current = Expr::apply((**head).clone(), new_args);
                    continue;
                }
            }

            if let Some((result, law_name)) = Self::apply_once(&current, laws) {
                if result != current {
                    proof.push(law_name);
                    current = result;
                    continue;
                }
            }
            break;
        }

        (current, proof)
    }

    pub fn innermost(expr: &Expr, laws: &[Law], max_steps: usize) -> (Expr, Vec<String>) {
        Self::bottom_up(expr, laws, max_steps)
    }

    pub fn outermost(expr: &Expr, laws: &[Law], max_steps: usize) -> (Expr, Vec<String>) {
        let mut interner = Interner::new();
        let mut proof = Vec::new();
        let mut current = expr.clone();
        let mut seen: HashSet<NodeId> = HashSet::new();

        for _ in 0..max_steps {
            if !seen.insert(interner.intern(&current)) {
                break;
            }
            if let Some((result, law_name)) = Self::apply_once(&current, laws) {
                if result != current {
                    proof.push(law_name);
                    current = result;
                    continue;
                }
            }
            if let Expr::Apply { head, args, .. } = &current {
                let mut new_args = Vec::with_capacity(args.len());
                let mut changed = false;
                for arg in args {
                    let (reduced, arg_proof) = Self::outermost(arg, laws, 1);
                    if !arg_proof.is_empty() {
                        proof.extend(arg_proof);
                        changed = true;
                    }
                    new_args.push(reduced);
                }
                if changed {
                    current = Expr::apply((**head).clone(), new_args);
                    continue;
                }
            }
            break;
        }
        (current, proof)
    }

    pub fn until_stable(expr: &Expr, laws: &[Law], max_steps: usize) -> (Expr, Vec<String>) {
        Self::bottom_up(expr, laws, max_steps)
    }

    pub fn limit(expr: &Expr, laws: &[Law], n: usize) -> (Expr, Vec<String>) {
        Self::bottom_up(expr, laws, n)
    }

    pub fn apply_innermost_once(expr: &Expr, laws: &[Law]) -> Option<(Expr, String)> {
        if let Expr::Apply { head, args, .. } = expr {
            for (index, arg) in args.iter().enumerate() {
                if let Some((reduced, law_name)) = Self::apply_innermost_once(arg, laws) {
                    let mut rebuilt = args.clone();
                    rebuilt[index] = reduced;
                    return Some((Expr::apply((**head).clone(), rebuilt), law_name));
                }
            }
            if let Some((result, law_name)) = Self::apply_once(expr, laws) {
                if result != *expr {
                    return Some((result, law_name));
                }
            }
        }
        None
    }

    pub fn reduce_steps(
        expr: &Expr,
        laws: &[Law],
        max_steps: usize,
    ) -> (Expr, Vec<(String, Expr, Expr)>) {
        let mut interner = Interner::new();
        let mut steps = Vec::new();
        let mut current = expr.clone();
        let mut seen: HashSet<NodeId> = HashSet::new();
        for _ in 0..max_steps {
            if !seen.insert(interner.intern(&current)) {
                break;
            }
            match Self::apply_innermost_once(&current, laws) {
                Some((next, law_name)) if next != current => {
                    steps.push((law_name, current.clone(), next.clone()));
                    current = next;
                }
                _ => break,
            }
        }
        (current, steps)
    }
}
