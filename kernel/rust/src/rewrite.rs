use std::collections::HashSet;

use crate::ac::canonicalize;
use crate::expr::Expr;
use crate::intern::{Interner, NodeId};
use crate::pattern::{match_expr, substitute, Bindings};
use crate::proof::{ProofObject, Reduction, Strategy};
use crate::space::{Law, Space};

pub struct RewriteEngine;

impl RewriteEngine {
    pub fn apply_once(expr: &Expr, laws: &[Law]) -> Option<(Expr, String)> {
        Self::apply_once_proof(expr, laws).map(|(expr, proof)| (expr, proof.law_id))
    }

    pub fn apply_once_proof(expr: &Expr, laws: &[Law]) -> Option<(Expr, ProofObject)> {
        for law in laws {
            if let Some(bindings) = match_expr(&law.pattern, expr, &Bindings::new()) {
                let result = substitute(&law.replacement, &bindings);
                if result != *expr {
                    return Some((
                        result.clone(),
                        ProofObject::law(law.name.clone(), &bindings, expr.clone(), result),
                    ));
                }
            }
        }
        None
    }

    pub fn reduce(expr: &Expr, laws: &[Law], max_steps: usize, strategy: Strategy) -> Reduction {
        let mut interner = Interner::new();
        let (result, steps, cycled) = match strategy {
            Strategy::BottomUp => Self::bottom_up_with(&mut interner, expr, laws, max_steps),
            Strategy::Innermost => Self::innermost_with(&mut interner, expr, laws, max_steps),
            Strategy::Outermost => Self::outermost_with(&mut interner, expr, laws, max_steps),
        };
        Reduction {
            result,
            steps,
            cycled,
            strategy,
        }
    }

    pub fn reduce_in_space(
        expr: &Expr,
        space: &Space,
        max_steps: usize,
        strategy: Strategy,
    ) -> Reduction {
        let mut reduction = Self::reduce(expr, &space.laws, max_steps, strategy);
        let canonical = canonicalize(&reduction.result, space);
        if canonical != reduction.result {
            reduction.steps.push(ProofObject::law(
                format!("ac.canonicalize:{}", space.name),
                &Bindings::new(),
                reduction.result.clone(),
                canonical.clone(),
            ));
            reduction.result = canonical;
        }
        reduction
    }

    pub fn bottom_up(expr: &Expr, laws: &[Law], max_steps: usize) -> (Expr, Vec<String>) {
        let reduction = Self::reduce(expr, laws, max_steps, Strategy::BottomUp);
        let names = reduction.law_names();
        (reduction.result, names)
    }

    pub fn innermost(expr: &Expr, laws: &[Law], max_steps: usize) -> (Expr, Vec<String>) {
        let reduction = Self::reduce(expr, laws, max_steps, Strategy::Innermost);
        let names = reduction.law_names();
        (reduction.result, names)
    }

    pub fn outermost(expr: &Expr, laws: &[Law], max_steps: usize) -> (Expr, Vec<String>) {
        let reduction = Self::reduce(expr, laws, max_steps, Strategy::Outermost);
        let names = reduction.law_names();
        (reduction.result, names)
    }

    pub fn until_stable(expr: &Expr, laws: &[Law], max_steps: usize) -> (Expr, Vec<String>) {
        Self::bottom_up(expr, laws, max_steps)
    }

    pub fn limit(expr: &Expr, laws: &[Law], n: usize) -> (Expr, Vec<String>) {
        Self::bottom_up(expr, laws, n)
    }

    fn bottom_up_with(
        interner: &mut Interner,
        expr: &Expr,
        laws: &[Law],
        max_steps: usize,
    ) -> (Expr, Vec<ProofObject>, bool) {
        let mut proof = Vec::new();
        let mut current = expr.clone();
        let mut seen: HashSet<NodeId> = HashSet::new();
        let mut used = 0usize;
        let mut cycled = false;

        while used < max_steps {
            if !seen.insert(interner.intern(&current)) {
                cycled = true;
                break;
            }

            if let Expr::Apply { head, args, .. } = &current {
                let mut new_args = Vec::with_capacity(args.len());
                let mut child_proofs = Vec::new();
                for arg in args {
                    let remain = max_steps - used;
                    let (reduced, arg_proof, arg_cycled) =
                        Self::bottom_up_with(interner, arg, laws, remain);
                    used = used.saturating_add(arg_proof.len());
                    child_proofs.extend(arg_proof);
                    if arg_cycled {
                        cycled = true;
                    }
                    new_args.push(reduced);
                    if used >= max_steps {
                        break;
                    }
                }
                if !child_proofs.is_empty() {
                    current = Expr::apply(head.as_ref().clone(), new_args);
                    proof.extend(child_proofs);
                    continue;
                }
            }

            if let Some((result, law_proof)) = Self::apply_once_proof(&current, laws) {
                proof.push(law_proof);
                current = result;
                used += 1;
                continue;
            }
            break;
        }

        (current, proof, cycled)
    }

    fn innermost_with(
        interner: &mut Interner,
        expr: &Expr,
        laws: &[Law],
        max_steps: usize,
    ) -> (Expr, Vec<ProofObject>, bool) {
        let mut proof = Vec::new();
        let mut current = expr.clone();
        let mut seen: HashSet<NodeId> = HashSet::new();
        let mut cycled = false;

        for _ in 0..max_steps {
            if !seen.insert(interner.intern(&current)) {
                cycled = true;
                break;
            }
            match Self::apply_innermost_once_proof(&current, laws) {
                Some((next, step)) => {
                    proof.push(step);
                    current = next;
                }
                None => break,
            }
        }
        (current, proof, cycled)
    }

    fn outermost_with(
        interner: &mut Interner,
        expr: &Expr,
        laws: &[Law],
        max_steps: usize,
    ) -> (Expr, Vec<ProofObject>, bool) {
        let mut proof = Vec::new();
        let mut current = expr.clone();
        let mut seen: HashSet<NodeId> = HashSet::new();
        let mut cycled = false;

        for _ in 0..max_steps {
            if !seen.insert(interner.intern(&current)) {
                cycled = true;
                break;
            }
            if let Some((result, step)) = Self::apply_once_proof(&current, laws) {
                proof.push(step);
                current = result;
                continue;
            }
            if let Expr::Apply { head, args, .. } = &current {
                let mut new_args = args.clone();
                let mut changed = false;
                for (index, arg) in args.iter().enumerate() {
                    let (reduced, arg_proof, arg_cycled) =
                        Self::outermost_with(interner, arg, laws, 1);
                    if arg_cycled {
                        cycled = true;
                    }
                    if !arg_proof.is_empty() {
                        new_args[index] = reduced;
                        proof.extend(arg_proof);
                        changed = true;
                        break;
                    }
                }
                if changed {
                    current = Expr::apply(head.as_ref().clone(), new_args);
                    continue;
                }
            }
            break;
        }
        (current, proof, cycled)
    }

    pub fn apply_innermost_once(expr: &Expr, laws: &[Law]) -> Option<(Expr, String)> {
        Self::apply_innermost_once_proof(expr, laws).map(|(expr, proof)| (expr, proof.law_id))
    }

    fn apply_innermost_once_proof(expr: &Expr, laws: &[Law]) -> Option<(Expr, ProofObject)> {
        if let Expr::Apply { head, args, .. } = expr {
            for (index, arg) in args.iter().enumerate() {
                if let Some((reduced, proof)) = Self::apply_innermost_once_proof(arg, laws) {
                    let mut rebuilt = args.clone();
                    rebuilt[index] = reduced;
                    return Some((Expr::apply(head.as_ref().clone(), rebuilt), proof));
                }
            }
        }
        Self::apply_once_proof(expr, laws)
    }

    pub fn reduce_steps(
        expr: &Expr,
        laws: &[Law],
        max_steps: usize,
    ) -> (Expr, Vec<(String, Expr, Expr)>) {
        let reduction = Self::reduce(expr, laws, max_steps, Strategy::Innermost);
        let steps = reduction
            .steps
            .iter()
            .map(|step| (step.law_id.clone(), step.before.clone(), step.after.clone()))
            .collect();
        (reduction.result, steps)
    }
}
