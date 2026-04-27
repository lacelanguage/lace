use std::sync::Arc;
use crate::inst::Inst;

pub struct Function {
    pub bytecode: Arc<[Inst]>,
    pub ip: usize,
}

impl Function {
    pub fn new(bytecode: &[Inst]) -> Self {
        Self {
            bytecode: bytecode.into(),
            ip: 0
        }
    }
}