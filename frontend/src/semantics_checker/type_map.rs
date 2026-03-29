use super::ty::Type;
use crate::parser::ast::NodeId;
use std::collections::HashMap;

pub struct TypeMap {
    inner: HashMap<NodeId, Type>
}

impl TypeMap {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new()
        }
    }
    
    pub fn assign_type(&mut self, id: NodeId, ty: Type) {
        self.inner.insert(id, ty);
    }

    pub fn get_type(&self, id: NodeId) -> Option<&Type> {
        self.inner.get(&id)
    }
}