use std::fmt;
use std::hash::{Hash, Hasher};

use crate::types::Type;

#[derive(Clone, Debug)]
pub enum AtomValue {
    Int(i64),
    Float(f64),
    Str(String),
}

impl PartialEq for AtomValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Int(a), Self::Int(b)) => a == b,
            (Self::Float(a), Self::Float(b)) => a == b,
            (Self::Str(a), Self::Str(b)) => a == b,
            _ => false,
        }
    }
}

impl Hash for AtomValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Self::Int(v) => v.hash(state),
            Self::Float(v) => canonical_float_bits(*v).hash(state),
            Self::Str(v) => v.hash(state),
        }
    }
}

pub(crate) fn canonical_float_bits(value: f64) -> u64 {
    if value.is_nan() {
        f64::NAN.to_bits()
    } else if value == 0.0 {
        0.0f64.to_bits()
    } else {
        value.to_bits()
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Expr {
    Symbol {
        name: String,
        ty: Option<Type>,
    },
    Atom {
        value: AtomValue,
        ty: Option<Type>,
    },
    Variable {
        name: String,
        ty: Option<Type>,
    },
    Apply {
        head: Box<Expr>,
        args: Vec<Expr>,
        ty: Option<Type>,
    },
}

impl Expr {
    pub fn symbol(name: impl Into<String>, ty: Option<Type>) -> Self {
        Expr::Symbol {
            name: name.into(),
            ty,
        }
    }

    pub fn atom(value: AtomValue, ty: Option<Type>) -> Self {
        Expr::Atom { value, ty }
    }

    pub fn var(name: impl Into<String>, ty: Option<Type>) -> Self {
        Expr::Variable {
            name: name.into(),
            ty,
        }
    }

    pub fn apply(head: Expr, args: Vec<Expr>) -> Self {
        let ty = infer_apply_type(&head, &args);
        Expr::Apply {
            head: Box::new(head),
            args,
            ty,
        }
    }

    pub fn ty(&self) -> Option<&Type> {
        match self {
            Expr::Symbol { ty, .. }
            | Expr::Atom { ty, .. }
            | Expr::Variable { ty, .. }
            | Expr::Apply { ty, .. } => ty.as_ref(),
        }
    }

    pub fn name(&self) -> Option<&str> {
        match self {
            Expr::Symbol { name, .. } | Expr::Variable { name, .. } => Some(name),
            _ => None,
        }
    }
}

pub(crate) fn infer_apply_type(head: &Expr, args: &[Expr]) -> Option<Type> {
    let mut current = head.ty()?.clone();
    if args.is_empty() {
        return Some(current);
    }
    for arg in args {
        match current {
            Type::Arrow(domain, codomain) => {
                if let Some(arg_ty) = arg.ty() {
                    if *domain != *arg_ty {
                        return None;
                    }
                }
                current = *codomain;
            }
            _ => return None,
        }
    }
    Some(current)
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Symbol { name, .. } | Expr::Variable { name, .. } => write!(f, "{name}"),
            Expr::Atom { value, .. } => match value {
                AtomValue::Int(v) => write!(f, "{v}"),
                AtomValue::Float(v) => write!(f, "{v}"),
                AtomValue::Str(v) => write!(f, "{v:?}"),
            },
            Expr::Apply { head, args, .. } => {
                write!(f, "{head}(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{arg}")?;
                }
                write!(f, ")")
            }
        }
    }
}
