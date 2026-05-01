use std::fmt;
use super::ss::SlotId;
use super::function::Function;
use lace_span::Span;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstantId(pub usize);

impl fmt::Debug for ConstantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "c{}", self.0)
    }
}


#[derive(Clone, PartialEq)]
pub enum IrValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    Unit,
    Tuple(Vec<IrValue>),
}

impl fmt::Debug for IrValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(n) => write!(f, "int(0x{n:016X})"),
            Self::Float(n) => write!(f, "float(0x{:016X})", n.to_bits()),
            Self::Bool(b) => write!(f, "bool({})", b.then(|| 1).unwrap_or(0)),
            Self::Unit => write!(f, "unit"),
            Self::Tuple(items) => write!(f,
                "tuple({})",
                items.iter()
                    .fold(
                        String::new(),
                        |acc, i| if acc.is_empty() {
                            format!("{i:?}")
                        } else {
                            format!("{acc}, {i:?}")
                        }
                    )
            ),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Register(pub usize);

impl fmt::Debug for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "r{}", self.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueId {
    Register(Register),
    Constant(ConstantId),
    StackSlot(SlotId),
}

impl fmt::Debug for ValueId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Register(r) => r.fmt(f),
            Self::Constant(c) => c.fmt(f),
            Self::StackSlot(s) => s.fmt(f),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum InstKind {
    Mov(Register, ValueId),
    IAdd(Register, ValueId, ValueId),
    ISub(Register, ValueId, ValueId),
    IMul(Register, ValueId, ValueId),
    IDiv(Register, ValueId, ValueId),
    IRem(Register, ValueId, ValueId),
    IPow(Register, ValueId, ValueId),
    FAdd(Register, ValueId, ValueId),
    FSub(Register, ValueId, ValueId),
    FMul(Register, ValueId, ValueId),
    FDiv(Register, ValueId, ValueId),
    FRem(Register, ValueId, ValueId),
    FPow(Register, ValueId, ValueId),
    MakeTuple(Register, Vec<ValueId>),
    StoreSS(SlotId, ValueId),
    Ret(ValueId),
}

impl fmt::Debug for InstKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mov(r, c) => write!(f, "{r:?} = {c:?}"),
            Self::IAdd(d, s1, s2) => write!(f, "{d:?} = iadd {s1:?}, {s2:?}"),
            Self::ISub(d, s1, s2) => write!(f, "{d:?} = isub {s1:?}, {s2:?}"),
            Self::IMul(d, s1, s2) => write!(f, "{d:?} = imul {s1:?}, {s2:?}"),
            Self::IDiv(d, s1, s2) => write!(f, "{d:?} = idiv {s1:?}, {s2:?}"),
            Self::IRem(d, s1, s2) => write!(f, "{d:?} = irem {s1:?}, {s2:?}"),
            Self::IPow(d, s1, s2) => write!(f, "{d:?} = ipow {s1:?}, {s2:?}"),
            Self::FAdd(d, s1, s2) => write!(f, "{d:?} = fadd {s1:?}, {s2:?}"),
            Self::FSub(d, s1, s2) => write!(f, "{d:?} = fsub {s1:?}, {s2:?}"),
            Self::FMul(d, s1, s2) => write!(f, "{d:?} = fmul {s1:?}, {s2:?}"),
            Self::FDiv(d, s1, s2) => write!(f, "{d:?} = fdiv {s1:?}, {s2:?}"),
            Self::FRem(d, s1, s2) => write!(f, "{d:?} = frem {s1:?}, {s2:?}"),
            Self::FPow(d, s1, s2) => write!(f, "{d:?} = fpow {s1:?}, {s2:?}"),
            Self::MakeTuple(d, srcs) => write!(
                f, "{d:?} = make_tuple({})",
                srcs.iter()
                    .fold(
                        String::new(),
                        |acc, v| if acc.is_empty() {
                            format!("{v:?}")
                        } else {
                            format!("{acc}, {v:?}")
                        }
                    )
            ),
            Self::StoreSS(ss, src) => write!(f, "store_ss {ss:?}, {src:?}"),
            Self::Ret(r) => write!(f, "ret {r:?}"),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Inst {
    pub kind: InstKind,
    pub span: Span
}

impl fmt::Debug for Inst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
    }
}

pub struct InstBuilder<'a> {
    f: &'a mut Function,
    span: Span,
}

impl<'a> InstBuilder<'a> {
    pub fn new(f: &'a mut Function, span: Span) -> Self {
        Self { f, span }
    }

    pub fn mov(self, r: ValueId) -> Register {
        if let Some(block) = self.f.current_block {
            let reg = self.f.allocate_register();
            self.f.blocks[block.0].insts.push(Inst { kind: InstKind::Mov(reg, r), span: self.span });
            reg
        } else {
            panic!("No block selected");
        }
    }

    pub fn iadd(self, l: ValueId, r: ValueId) -> Register {
        if let Some(block) = self.f.current_block {
            let reg = self.f.allocate_register();
            self.f.blocks[block.0].insts.push(Inst { kind: InstKind::IAdd(reg, l, r), span: self.span });
            reg
        } else {
            panic!("No block selected");
        }
    }

    pub fn isub(self, l: ValueId, r: ValueId) -> Register {
        if let Some(block) = self.f.current_block {
            let reg = self.f.allocate_register();
            self.f.blocks[block.0].insts.push(Inst { kind: InstKind::ISub(reg, l, r), span: self.span });
            reg
        } else {
            panic!("No block selected");
        }
    }

    pub fn imul(self, l: ValueId, r: ValueId) -> Register {
        if let Some(block) = self.f.current_block {
            let reg = self.f.allocate_register();
            self.f.blocks[block.0].insts.push(Inst { kind: InstKind::IMul(reg, l, r), span: self.span });
            reg
        } else {
            panic!("No block selected");
        }
    }

    pub fn idiv(self, l: ValueId, r: ValueId) -> Register {
        if let Some(block) = self.f.current_block {
            let reg = self.f.allocate_register();
            self.f.blocks[block.0].insts.push(Inst { kind: InstKind::IDiv(reg, l, r), span: self.span });
            reg
        } else {
            panic!("No block selected");
        }
    }

    pub fn irem(self, l: ValueId, r: ValueId) -> Register {
        if let Some(block) = self.f.current_block {
            let reg = self.f.allocate_register();
            self.f.blocks[block.0].insts.push(Inst { kind: InstKind::IRem(reg, l, r), span: self.span });
            reg
        } else {
            panic!("No block selected");
        }
    }

    pub fn ipow(self, l: ValueId, r: ValueId) -> Register {
        if let Some(block) = self.f.current_block {
            let reg = self.f.allocate_register();
            self.f.blocks[block.0].insts.push(Inst { kind: InstKind::IPow(reg, l, r), span: self.span });
            reg
        } else {
            panic!("No block selected");
        }
    }

    pub fn fadd(self, l: ValueId, r: ValueId) -> Register {
        if let Some(block) = self.f.current_block {
            let reg = self.f.allocate_register();
            self.f.blocks[block.0].insts.push(Inst { kind: InstKind::FAdd(reg, l, r), span: self.span });
            reg
        } else {
            panic!("No block selected");
        }
    }

    pub fn fsub(self, l: ValueId, r: ValueId) -> Register {
        if let Some(block) = self.f.current_block {
            let reg = self.f.allocate_register();
            self.f.blocks[block.0].insts.push(Inst { kind: InstKind::FSub(reg, l, r), span: self.span });
            reg
        } else {
            panic!("No block selected");
        }
    }

    pub fn fmul(self, l: ValueId, r: ValueId) -> Register {
        if let Some(block) = self.f.current_block {
            let reg = self.f.allocate_register();
            self.f.blocks[block.0].insts.push(Inst { kind: InstKind::FMul(reg, l, r), span: self.span });
            reg
        } else {
            panic!("No block selected");
        }
    }

    pub fn fdiv(self, l: ValueId, r: ValueId) -> Register {
        if let Some(block) = self.f.current_block {
            let reg = self.f.allocate_register();
            self.f.blocks[block.0].insts.push(Inst { kind: InstKind::FDiv(reg, l, r), span: self.span });
            reg
        } else {
            panic!("No block selected");
        }
    }

    pub fn frem(self, l: ValueId, r: ValueId) -> Register {
        if let Some(block) = self.f.current_block {
            let reg = self.f.allocate_register();
            self.f.blocks[block.0].insts.push(Inst { kind: InstKind::FRem(reg, l, r), span: self.span });
            reg
        } else {
            panic!("No block selected");
        }
    }

    pub fn fpow(self, l: ValueId, r: ValueId) -> Register {
        if let Some(block) = self.f.current_block {
            let reg = self.f.allocate_register();
            self.f.blocks[block.0].insts.push(Inst { kind: InstKind::FPow(reg, l, r), span: self.span });
            reg
        } else {
            panic!("No block selected");
        }
    }

    pub fn make_tuple(self, srcs: Vec<ValueId>) -> Register {
        if let Some(block) = self.f.current_block {
            let reg = self.f.allocate_register();
            self.f.blocks[block.0].insts.push(Inst { kind: InstKind::MakeTuple(reg, srcs), span: self.span });
            reg
        } else {
            panic!("No block selected");
        }
    }

    pub fn store_ss(self, ss: SlotId, src: ValueId) {
        if let Some(block) = self.f.current_block {
            self.f.blocks[block.0].insts.push(Inst { kind: InstKind::StoreSS(ss, src), span: self.span });
        } else {
            panic!("No block selected");
        }
    }

    pub fn ret(self, r: ValueId) {
        if let Some(block) = self.f.current_block {
            self.f.blocks[block.0].insts.push(Inst { kind: InstKind::Ret(r), span: self.span });
        } else {
            panic!("No block selected");
        }
    }
}