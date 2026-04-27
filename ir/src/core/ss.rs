use std::fmt;
use super::ty::Type;

#[derive(Clone, PartialEq)]
pub struct StackSlot {
    pub id: SlotId,
    pub slot_ty: Type,
}

impl fmt::Debug for StackSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {:?}", self.id, self.slot_ty)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SlotId(pub usize);

impl fmt::Debug for SlotId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ss{}", self.0)
    }
}