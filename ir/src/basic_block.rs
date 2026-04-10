use std::fmt;
use super::inst::*;

#[derive(Clone, PartialEq)]
pub struct Block {
    pub id: BlockId,
    pub insts: Vec<Inst>
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub usize);

impl fmt::Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ".BB{}:{}", self.id.0,
            self.insts.iter()
                .fold(
                    String::new(),
                    |acc, inst| format!("{acc}\n\t{inst:?}")
                )
        )
    }
}