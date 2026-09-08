//! Universal Mathematical Kernel.
//!
//! The kernel knows nothing about mathematics. It only represents expressions,
//! matches patterns, applies rewrite laws, and checks types.
//!
//! Kernel != Mathematics
//! Kernel + Laws = Mathematical System

pub mod ac;
pub mod expr;
pub mod intern;
pub mod pattern;
pub mod proof;
pub mod rewrite;
pub mod semantics;
pub mod space;
pub mod typecheck;
pub mod types;

pub use ac::{ac_eq, canonicalize};
pub use expr::{AtomValue, Expr};
pub use intern::Interner;
pub use pattern::{match_expr, substitute};
pub use proof::{
    replay, verify, verify_objects, Instantiation, ProofObject, ProofStep, Reduction, Strategy,
};
pub use rewrite::RewriteEngine;
pub use semantics::{interpret, Environment, SemanticValue};
pub use space::{Law, OpProperty, Space};
pub use typecheck::{type_check, Judgement, Phase, TypeCheckError};
pub use types::Type;
