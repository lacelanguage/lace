use std::collections::{HashMap, HashSet};
use super::Optimization;
use crate::core::{
    basic_block::{Block, BlockId},
    function::Function,
    inst::*,
    module::Module,
    ss::SlotId
};

pub struct DeadCodeElimination {
    pub defined_registers: HashMap<Register, (usize, usize)>,
    pub used_registers: HashSet<Register>,
    pub used_blocks: HashSet<BlockId>,
    pub used_constants: HashSet<ConstantId>,
    pub defined_stack_slots: HashMap<SlotId, (usize, usize)>,
    pub used_stack_slots: HashSet<SlotId>
}

impl Optimization for DeadCodeElimination {
    fn new() -> Self {
        Self {
            defined_registers: HashMap::new(),
            used_registers: HashSet::new(),
            used_blocks: HashSet::new(),
            used_constants: HashSet::new(),
            defined_stack_slots: HashMap::new(),
            used_stack_slots: HashSet::new(),
        }
    }

    fn apply(&mut self, ir: &mut Module) {
        for function in &mut ir.functions {
            self.apply_func(function);

            self.defined_registers.clear();
            self.used_registers.clear();
            self.used_blocks.clear();
            self.used_constants.clear();
            self.defined_stack_slots.clear();
            self.used_stack_slots.clear();
        }
    }
}

impl DeadCodeElimination {
    fn apply_func(&mut self, func: &mut Function) {
        if func.blocks.len() < 1 { return }

        let mut insts = func.blocks.clone()
            .into_iter()
            .map(|b| b.insts.into_iter().map(|i| Some(i)).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let mut removed_blocks = vec![];

        for (idx, block) in func.blocks.iter_mut().enumerate() {
            self.apply_block(block, idx);
        }

        for (reg, (block, inst)) in &self.defined_registers {
            if self.used_registers.get(&reg).is_none() {
                insts[*block][*inst] = None;
            }
        }

        for (ss, (block, inst)) in &self.defined_stack_slots {
            if self.used_stack_slots.get(&ss).is_none() {
                insts[*block][*inst] = None;
            }
        }
        func.stack_slots.retain(|ss| self.used_stack_slots.get(&ss.id).is_some());

        for block in 1..func.blocks.len() {
            if self.used_blocks.get(&BlockId(block)).is_none() {
                removed_blocks.push(BlockId(block));
            }
        }

        for (idx, block_insts) in insts.into_iter().enumerate() {
            func.blocks[idx].insts = block_insts.into_iter()
                .filter(|i| i.is_some())
                .map(|i| i.unwrap())
                .collect();
        }

        func.blocks.retain(|b| !removed_blocks.contains(&b.id));

        self.used_constants.clear();

        for (idx, block) in func.blocks.iter_mut().enumerate() {
            self.apply_block(block, idx);
        }

        func.constants.retain(|c| self.used_constants.get(&c.id).is_some());
    }

    fn apply_block(&mut self, block: &mut Block, block_idx: usize) {
        for (idx, inst) in block.insts.iter().enumerate() {
            match &inst.kind {
                InstKind::Mov(r, s) => {
                    self.defined_registers.insert(*r, (block_idx, idx));
                    match s {
                        ValueId::Constant(c) => self.used_constants.insert(*c),
                        ValueId::Register(s) => self.used_registers.insert(*s),
                        ValueId::StackSlot(s) => self.used_stack_slots.insert(*s)
                    };
                },
                InstKind::IAdd(d, src1, src2)
                | InstKind::ISub(d, src1, src2)
                | InstKind::IMul(d, src1, src2)
                | InstKind::IDiv(d, src1, src2)
                | InstKind::IRem(d, src1, src2)
                | InstKind::IPow(d, src1, src2)
                | InstKind::FAdd(d, src1, src2)
                | InstKind::FSub(d, src1, src2)
                | InstKind::FMul(d, src1, src2)
                | InstKind::FDiv(d, src1, src2)
                | InstKind::FRem(d, src1, src2)
                | InstKind::FPow(d, src1, src2)
                | InstKind::ICmp(_, d, src1, src2)
                | InstKind::FCmp(_, d, src1, src2) =>
                {
                    self.defined_registers.insert(*d, (block_idx, idx));
                    match src1 {
                        ValueId::Constant(c) => self.used_constants.insert(*c),
                        ValueId::Register(s) => self.used_registers.insert(*s),
                        ValueId::StackSlot(s) => self.used_stack_slots.insert(*s)
                    };
                    match src2 {
                        ValueId::Constant(c) => self.used_constants.insert(*c),
                        ValueId::Register(s) => self.used_registers.insert(*s),
                        ValueId::StackSlot(s) => self.used_stack_slots.insert(*s)
                    };
                },
                InstKind::MakeTuple(d, srcs) => {
                    self.defined_registers.insert(*d, (block_idx, idx));
                    for s in srcs {
                        match s {
                            ValueId::Constant(c) => self.used_constants.insert(*c),
                            ValueId::Register(s) => self.used_registers.insert(*s),
                            ValueId::StackSlot(s) => self.used_stack_slots.insert(*s)
                        };
                    }
                },
                InstKind::Jmp(b, b_args) => {
                    self.used_blocks.insert(*b);
                    for s in b_args {
                        match s {
                            ValueId::Constant(c) => self.used_constants.insert(*c),
                            ValueId::Register(s) => self.used_registers.insert(*s),
                            ValueId::StackSlot(s) => self.used_stack_slots.insert(*s)
                        };
                    }
                },
                InstKind::Brif(cond, then_b, then_args, else_b, else_args) => {
                    match cond {
                        ValueId::Constant(c) => self.used_constants.insert(*c),
                        ValueId::Register(s) => self.used_registers.insert(*s),
                        ValueId::StackSlot(s) => self.used_stack_slots.insert(*s)
                    };
                    self.used_blocks.insert(*then_b);
                    for s in then_args {
                        match s {
                            ValueId::Constant(c) => self.used_constants.insert(*c),
                            ValueId::Register(s) => self.used_registers.insert(*s),
                            ValueId::StackSlot(s) => self.used_stack_slots.insert(*s)
                        };
                    }
                    self.used_blocks.insert(*else_b);
                    for s in else_args {
                        match s {
                            ValueId::Constant(c) => self.used_constants.insert(*c),
                            ValueId::Register(s) => self.used_registers.insert(*s),
                            ValueId::StackSlot(s) => self.used_stack_slots.insert(*s)
                        };
                    }
                },
                InstKind::StoreSS(ss, s) => {
                    self.defined_stack_slots.insert(*ss, (block_idx, idx));
                    match s {
                        ValueId::Constant(c) => self.used_constants.insert(*c),
                        ValueId::Register(s) => self.used_registers.insert(*s),
                        ValueId::StackSlot(s) => self.used_stack_slots.insert(*s)
                    };
                },
                InstKind::Ret(s) => {
                    match s {
                        ValueId::Constant(c) => self.used_constants.insert(*c),
                        ValueId::Register(s) => self.used_registers.insert(*s),
                        ValueId::StackSlot(s) => self.used_stack_slots.insert(*s)
                    };
                },
            }
        }
    }
}