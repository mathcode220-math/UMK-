//! Universal Mathematical Kernel.
//!
//! The kernel knows nothing about mathematics. It only represents expressions,
//! matches patterns, applies rewrite laws, and checks types.

pub mod expr;
pub mod intern;
pub mod pattern;
pub mod proof;
pub mod rewrite;
pub mod space;
pub mod typecheck;
pub mod types;

pub use expr::{AtomValue, Expr};
pub use intern::Interner;
pub use pattern::{match_expr, substitute};
pub use proof::{replay, verify, ProofStep};
pub use rewrite::RewriteEngine;
pub use space::{Law, Space};
pub use typecheck::{type_check, TypeCheckError};
pub use types::Type;
