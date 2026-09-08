use crate::expr::Expr;
use crate::types::Type;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeCheckError(pub String);

impl std::fmt::Display for TypeCheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for TypeCheckError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Parsed,
    Typed,
    Validated,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Judgement {
    pub expr: Expr,
    pub phase: Phase,
    pub ty: Option<Type>,
}

impl Judgement {
    pub fn parsed(expr: Expr) -> Self {
        Judgement {
            expr,
            phase: Phase::Parsed,
            ty: None,
        }
    }

    pub fn typed(self) -> Result<Self, TypeCheckError> {
        let ty = type_check(&self.expr)?;
        Ok(Judgement {
            expr: self.expr,
            phase: Phase::Typed,
            ty,
        })
    }

    pub fn validated(self) -> Result<Self, TypeCheckError> {
        let typed = self.typed()?;
        let phase = if typed.ty.is_some() {
            Phase::Validated
        } else {
            Phase::Typed
        };
        Ok(Judgement { phase, ..typed })
    }
}

fn arrow_arity(ty: &Type) -> usize {
    match ty {
        Type::Arrow(_, codomain) => 1 + arrow_arity(codomain),
        _ => 0,
    }
}

pub fn type_check(expr: &Expr) -> Result<Option<Type>, TypeCheckError> {
    match expr {
        Expr::Symbol { ty, .. } | Expr::Variable { ty, .. } | Expr::Atom { ty, .. } => {
            Ok(ty.clone())
        }
        Expr::Apply { head, args, .. } => {
            let head_type = match type_check(head)? {
                Some(ty) => ty,
                None => return Ok(None),
            };
            if args.is_empty() {
                return Ok(Some(head_type));
            }
            let arity = arrow_arity(&head_type);
            if arity == 0 {
                return Err(TypeCheckError(format!(
                    "Head {head} has type {head_type}, expected Arrow"
                )));
            }
            if args.len() > arity {
                return Err(TypeCheckError(format!(
                    "Too many arguments: head {head} has arity {arity}, got {}",
                    args.len()
                )));
            }
            let mut current = head_type;
            for (index, arg) in args.iter().enumerate() {
                match current {
                    Type::Arrow(domain, codomain) => {
                        if let Some(arg_ty) = type_check(arg)? {
                            if *domain != arg_ty {
                                return Err(TypeCheckError(format!(
                                    "Type mismatch in argument {index} of {head}: expected {domain}, got {arg_ty}"
                                )));
                            }
                        }
                        current = *codomain;
                    }
                    other => {
                        return Err(TypeCheckError(format!(
                            "Head {head} has type {other}, expected Arrow"
                        )));
                    }
                }
            }
            Ok(Some(current))
        }
    }
}
