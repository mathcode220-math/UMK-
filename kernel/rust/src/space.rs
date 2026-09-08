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

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum OpProperty {
    Associative,
    Commutative,
}

impl OpProperty {
    pub fn as_str(self) -> &'static str {
        match self {
            OpProperty::Associative => "associative",
            OpProperty::Commutative => "commutative",
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Space {
    pub name: String,
    pub types: Vec<Type>,
    pub operations: Vec<Expr>,
    pub laws: Vec<Law>,
    pub properties: Vec<(String, OpProperty)>,
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
            properties: Vec::new(),
        }
    }

    pub fn with_properties(mut self, properties: Vec<(String, OpProperty)>) -> Self {
        self.properties = properties;
        self
    }

    pub fn declares(&self, op: &str, property: OpProperty) -> bool {
        self.properties
            .iter()
            .any(|(name, prop)| name == op && *prop == property)
    }

    pub fn op_name(expr: &Expr) -> Option<&str> {
        match expr {
            Expr::Symbol { name, .. } => Some(name),
            _ => None,
        }
    }
}
