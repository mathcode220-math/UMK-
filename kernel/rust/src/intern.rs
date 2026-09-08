use std::collections::HashMap;

use crate::expr::{canonical_float_bits, AtomValue, Expr};
use crate::types::Type;

pub type NodeId = u32;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum NodeKey {
    Symbol {
        name: String,
        ty: Option<Type>,
    },
    AtomInt {
        value: i64,
        ty: Option<Type>,
    },
    AtomFloat {
        bits: u64,
        ty: Option<Type>,
    },
    AtomStr {
        value: String,
        ty: Option<Type>,
    },
    Variable {
        name: String,
        ty: Option<Type>,
    },
    Apply {
        head: NodeId,
        args: Vec<NodeId>,
    },
}

#[derive(Default)]
pub struct Interner {
    keys: HashMap<NodeKey, NodeId>,
    nodes: Vec<Expr>,
}

impl Interner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn intern(&mut self, expr: &Expr) -> NodeId {
        let key = match expr {
            Expr::Symbol { name, ty } => NodeKey::Symbol {
                name: name.clone(),
                ty: ty.clone(),
            },
            Expr::Atom {
                value: AtomValue::Int(value),
                ty,
            } => NodeKey::AtomInt {
                value: *value,
                ty: ty.clone(),
            },
            Expr::Atom {
                value: AtomValue::Float(value),
                ty,
            } => NodeKey::AtomFloat {
                bits: canonical_float_bits(*value),
                ty: ty.clone(),
            },
            Expr::Atom {
                value: AtomValue::Str(value),
                ty,
            } => NodeKey::AtomStr {
                value: value.clone(),
                ty: ty.clone(),
            },
            Expr::Variable { name, ty } => NodeKey::Variable {
                name: name.clone(),
                ty: ty.clone(),
            },
            Expr::Apply { head, args, .. } => NodeKey::Apply {
                head: self.intern(head),
                args: args.iter().map(|arg| self.intern(arg)).collect(),
            },
        };
        if let Some(id) = self.keys.get(&key) {
            return *id;
        }
        let id = self.nodes.len() as NodeId;
        self.keys.insert(key, id);
        self.nodes.push(expr.clone());
        id
    }

    pub fn get(&self, id: NodeId) -> Option<&Expr> {
        self.nodes.get(id as usize)
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}
