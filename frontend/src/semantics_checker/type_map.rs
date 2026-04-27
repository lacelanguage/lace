use lace_span::Span;
use super::ty::Type;
use crate::parser::ast::{NodeId, FuncId};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDefTypeInfo {
    pub params: Vec<Type>,
    pub return_ty: Type,
    pub defined_at: Span
}

pub struct TypeMap {
    nodes: HashMap<NodeId, Type>,
    funcs: HashMap<FuncId, FunctionDefTypeInfo>,
}

impl TypeMap {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            funcs: HashMap::new()
        }
    }
    
    pub fn assign_node(&mut self, id: NodeId, ty: Type) {
        self.nodes.insert(id, ty);
    }

    pub fn get_node(&self, id: NodeId) -> Option<&Type> {
        self.nodes.get(&id)
    }
    
    pub fn assign_func(&mut self, id: FuncId, ty: FunctionDefTypeInfo) {
        self.funcs.insert(id, ty);
    }

    pub fn get_func(&self, id: FuncId) -> Option<&FunctionDefTypeInfo> {
        self.funcs.get(&id)
    }
}