use std::fmt;
use super::ty::Type;
use super::inst::*;

#[derive(Clone, PartialEq)]
pub struct Block {
    pub id: BlockId,
    pub params: Vec<(Register, Type)>,
    pub insts: Vec<Inst>
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub usize);

impl fmt::Debug for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ".BB{}", self.0)
    }
}

impl fmt::Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}({}):{}", self.id,
            self.params.iter()
                .fold(
                    String::new(),
                    |acc, (r, t)| if acc.is_empty() {
                        format!("{r:?}: {t:?}")
                    } else {
                        format!("{acc}, {r:?}: {t:?}")
                    }
                ),
            self.insts.iter()
                .fold(
                    String::new(),
                    |acc, inst| format!("{acc}\n\t{inst:?}")
                )
        )
    }
}