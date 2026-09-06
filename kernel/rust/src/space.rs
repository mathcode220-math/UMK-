use crate::expr::Expr;
use crate::types::Type;

#[derive(Clone, PartialEq, Debug)]
pub struct Law {
    pub name: String,
    pub pattern: Expr,
    pub replacement: Expr,
}

impl Law {
    pub fn new(name: impl Into<String>, pattern: Expr, replacement: Expr) -> Self {
        Law {
            name: name.into(),
            pattern,
            replacement,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Space {
    pub name: String,
    pub types: Vec<Type>,
    pub operations: Vec<Expr>,
    pub laws: Vec<Law>,
}

impl Space {
    pub fn new(
        name: impl Into<String>,
        types: Vec<Type>,
        operations: Vec<Expr>,
        laws: Vec<Law>,
    ) -> Self {
        Space {
            name: name.into(),
            types,
            operations,
            laws,
        }
    }
}
