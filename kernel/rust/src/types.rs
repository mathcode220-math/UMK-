use std::fmt;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Type {
    Atom(String),
    Abstract(String),
    Arrow(Box<Type>, Box<Type>),
}

impl Type {
    pub fn atom(name: impl Into<String>) -> Self {
        Type::Atom(name.into())
    }

    pub fn abstract_carrier(name: impl Into<String>) -> Self {
        Type::Abstract(name.into())
    }

    pub fn arrow(domain: Type, codomain: Type) -> Self {
        Type::Arrow(Box::new(domain), Box::new(codomain))
    }

    pub fn compatible(&self, other: &Type) -> bool {
        self == other
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Atom(name) | Type::Abstract(name) => write!(f, "{name}"),
            Type::Arrow(domain, codomain) => write!(f, "({domain} -> {codomain})"),
        }
    }
}
