use std::collections::HashMap;
use super::Optimization;
use crate::core::{
    basic_block::Block,
    function::{Function, Constant},
    inst::*,
    module::Module
};

pub struct ConstantFolder {
    registers: HashMap<Register, IrValue>,
}

impl Optimization for ConstantFolder {
    fn new() -> Self {
        Self { registers: HashMap::new() }
    }

    fn apply(&mut self, ir: &mut Module) {
        for func in &mut ir.functions {
            self.apply_func(func);
            self.registers.clear();
        }
    }
}

impl ConstantFolder {
    fn apply_func(&mut self, func: &mut Function) {
        for block in func.blocks.iter_mut() {
            self.apply_block(block, &mut func.constants);
        }
    }

    fn apply_block(&mut self, block: &mut Block, constants: &mut Vec<Constant>) {
        for inst in block.insts.iter_mut() {
            match &mut inst.kind {
                InstKind::Mov(r, s) => match s {
                    ValueId::Constant(c) => {
                        self.registers.insert(*r, constants[c.0].inner.clone());
                    },
                    ValueId::Register(rs) => if let Some(val) = self.registers.get(rs).cloned() {
                        self.registers.insert(*r, val.clone());
                        let id = ConstantId(constants.len());
                        constants.push(Constant{ inner: val.clone(), id });
                        *s = ValueId::Constant(id);
                    },
                    ValueId::StackSlot(_) => ()
                },
                InstKind::IAdd(_, src1, src2)
                | InstKind::ISub(_, src1, src2)
                | InstKind::IMul(_, src1, src2)
                | InstKind::IDiv(_, src1, src2)
                | InstKind::IRem(_, src1, src2)
                | InstKind::IPow(_, src1, src2)
                | InstKind::FAdd(_, src1, src2)
                | InstKind::FSub(_, src1, src2)
                | InstKind::FMul(_, src1, src2)
                | InstKind::FDiv(_, src1, src2)
                | InstKind::FRem(_, src1, src2)
                | InstKind::FPow(_, src1, src2)
                | InstKind::ICmp(_, _, src1, src2)
                | InstKind::FCmp(_, _, src1, src2) => {
                    let lhs = match src1 {
                        ValueId::Constant(c) => Some(constants[c.0].inner.clone()),
                        ValueId::Register(s) => self.registers.get(&*s).cloned(),
                        ValueId::StackSlot(_) => None
                    };
                    let rhs = match src2 {
                        ValueId::Constant(c) => Some(constants[c.0].inner.clone()),
                        ValueId::Register(s) => self.registers.get(&*s).cloned(),
                        ValueId::StackSlot(_) => None
                    };

                    if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                        match &mut inst.kind {
                            InstKind::IAdd(d, _, _) => if let (IrValue::Int(lval), IrValue::Int(rval)) = (lhs, rhs) {
                                let result = IrValue::Int(lval + rval);
                                let id = ConstantId(constants.len());
                                constants.push(Constant { inner: result.clone(), id });
                                self.registers.insert(*d, result);
                                inst.kind = InstKind::Mov(*d, ValueId::Constant(id));
                            },
                            InstKind::ISub(d, _, _) => if let (IrValue::Int(lval), IrValue::Int(rval)) = (lhs, rhs) {
                                let result = IrValue::Int(lval - rval);
                                let id = ConstantId(constants.len());
                                constants.push(Constant { inner: result.clone(), id });
                                self.registers.insert(*d, result);
                                inst.kind = InstKind::Mov(*d, ValueId::Constant(id));
                            },
                            InstKind::IMul(d, _, _) => if let (IrValue::Int(lval), IrValue::Int(rval)) = (lhs, rhs) {
                                let result = IrValue::Int(lval * rval);
                                let id = ConstantId(constants.len());
                                constants.push(Constant { inner: result.clone(), id });
                                self.registers.insert(*d, result);
                                inst.kind = InstKind::Mov(*d, ValueId::Constant(id));
                            },
                            InstKind::IDiv(d, _, _) => if let (IrValue::Int(lval), IrValue::Int(rval)) = (lhs, rhs) {
                                let result = IrValue::Int(lval / rval);
                                let id = ConstantId(constants.len());
                                constants.push(Constant { inner: result.clone(), id });
                                self.registers.insert(*d, result);
                                inst.kind = InstKind::Mov(*d, ValueId::Constant(id));
                            },
                            InstKind::IRem(d, _, _) => if let (IrValue::Int(lval), IrValue::Int(rval)) = (lhs, rhs) {
                                let result = IrValue::Int(lval % rval);
                                let id = ConstantId(constants.len());
                                constants.push(Constant { inner: result.clone(), id });
                                self.registers.insert(*d, result);
                                inst.kind = InstKind::Mov(*d, ValueId::Constant(id));
                            },
                            InstKind::IPow(d, _, _) => if let (IrValue::Int(lval), IrValue::Int(rval)) = (lhs, rhs) {
                                let pow = rval;
                                if pow < 0 {
                                    let result = IrValue::Int(1 / lval.pow((pow.abs() % 64) as u32));
                                    let id = ConstantId(constants.len());
                                    constants.push(Constant { inner: result.clone(), id });
                                    self.registers.insert(*d, result);
                                    inst.kind = InstKind::Mov(*d, ValueId::Constant(id));
                                } else if pow > 0 {
                                    let result = IrValue::Int(lval.pow((pow as u64 % 64) as u32));
                                    let id = ConstantId(constants.len());
                                    constants.push(Constant { inner: result.clone(), id });
                                    self.registers.insert(*d, result);
                                    inst.kind = InstKind::Mov(*d, ValueId::Constant(id));
                                } else if lval != 0 {
                                    let id = ConstantId(constants.len());
                                    constants.push(Constant { inner: IrValue::Int(1), id });
                                    self.registers.insert(*d, IrValue::Int(1));
                                    inst.kind = InstKind::Mov(*d, ValueId::Constant(id));
                                }
                            },
                            InstKind::ICmp(fl, d, _, _) => if let (IrValue::Int(lval), IrValue::Int(rval)) = (lhs, rhs) {
                                let result = match fl {
                                    CmpFlag::Eq => IrValue::Bool(lval == rval),
                                    CmpFlag::Ne => IrValue::Bool(lval != rval),
                                    CmpFlag::Gt => IrValue::Bool(lval > rval),
                                    CmpFlag::Lt => IrValue::Bool(lval < rval),
                                    CmpFlag::Ge => IrValue::Bool(lval >= rval),
                                    CmpFlag::Le => IrValue::Bool(lval <= rval),
                                };
                                let id = ConstantId(constants.len());
                                constants.push(Constant { inner: result.clone(), id });
                                self.registers.insert(*d, result);
                                inst.kind = InstKind::Mov(*d, ValueId::Constant(id));
                            },
                            InstKind::FAdd(d, _, _) => if let (IrValue::Float(lval), IrValue::Float(rval)) = (lhs, rhs) {
                                let result = IrValue::Float(lval + rval);
                                let id = ConstantId(constants.len());
                                constants.push(Constant { inner: result.clone(), id });
                                self.registers.insert(*d, result);
                                inst.kind = InstKind::Mov(*d, ValueId::Constant(id));
                            },
                            InstKind::FSub(d, _, _) => if let (IrValue::Float(lval), IrValue::Float(rval)) = (lhs, rhs) {
                                let result = IrValue::Float(lval - rval);
                                let id = ConstantId(constants.len());
                                constants.push(Constant { inner: result.clone(), id });
                                self.registers.insert(*d, result);
                                inst.kind = InstKind::Mov(*d, ValueId::Constant(id));
                            },
                            InstKind::FMul(d, _, _) => if let (IrValue::Float(lval), IrValue::Float(rval)) = (lhs, rhs) {
                                let result = IrValue::Float(lval * rval);
                                let id = ConstantId(constants.len());
                                constants.push(Constant { inner: result.clone(), id });
                                self.registers.insert(*d, result);
                                inst.kind = InstKind::Mov(*d, ValueId::Constant(id));
                            },
                            InstKind::FDiv(d, _, _) => if let (IrValue::Float(lval), IrValue::Float(rval)) = (lhs, rhs) {
                                let result = IrValue::Float(lval / rval);
                                let id = ConstantId(constants.len());
                                constants.push(Constant { inner: result.clone(), id });
                                self.registers.insert(*d, result);
                                inst.kind = InstKind::Mov(*d, ValueId::Constant(id));
                            },
                            InstKind::FRem(d, _, _) => if let (IrValue::Float(lval), IrValue::Float(rval)) = (lhs, rhs) {
                                let result = IrValue::Float(lval % rval);
                                let id = ConstantId(constants.len());
                                constants.push(Constant { inner: result.clone(), id });
                                self.registers.insert(*d, result);
                                inst.kind = InstKind::Mov(*d, ValueId::Constant(id));
                            },
                            InstKind::FPow(d, _, _) => if let (IrValue::Float(lval), IrValue::Float(rval)) = (lhs, rhs) {
                                let result = IrValue::Float(lval.powf(rval));
                                let id = ConstantId(constants.len());
                                constants.push(Constant { inner: result.clone(), id });
                                self.registers.insert(*d, result);
                                inst.kind = InstKind::Mov(*d, ValueId::Constant(id));
                            },
                            InstKind::FCmp(fl, d, _, _) => if let (IrValue::Float(lval), IrValue::Float(rval)) = (lhs, rhs) {
                                let result = match fl {
                                    CmpFlag::Eq => IrValue::Bool(lval == rval),
                                    CmpFlag::Ne => IrValue::Bool(lval != rval),
                                    CmpFlag::Gt => IrValue::Bool(lval > rval),
                                    CmpFlag::Lt => IrValue::Bool(lval < rval),
                                    CmpFlag::Ge => IrValue::Bool(lval >= rval),
                                    CmpFlag::Le => IrValue::Bool(lval <= rval),
                                };
                                let id = ConstantId(constants.len());
                                constants.push(Constant { inner: result.clone(), id });
                                self.registers.insert(*d, result);
                                inst.kind = InstKind::Mov(*d, ValueId::Constant(id));
                            },
                            _ => unreachable!()
                        };
                    }
                },
                InstKind::Jmp(_b, _b_args) => (),
                InstKind::Brif(c, t, t_args, e, e_args) => {
                    let cond = match c {
                        ValueId::Constant(c) => Some(constants[c.0].inner.clone()),
                        ValueId::Register(s) => self.registers.get(&*s).cloned(),
                        ValueId::StackSlot(_) => None
                    };
                    if let Some(IrValue::Bool(true)) = cond {
                        inst.kind = InstKind::Jmp(*t, t_args.clone());
                    } else if let Some(IrValue::Bool(false)) = cond {
                        inst.kind = InstKind::Jmp(*e, e_args.clone());
                    }
                },
                InstKind::MakeTuple(d, srcs) => {
                    let mut tup = vec![];
                    let mut finish = true;

                    for src in srcs {
                        match src {
                            ValueId::Constant(c) => tup.push(constants[c.0].inner.clone()),
                            ValueId::Register(s) => if let Some(val) = self.registers.get(&*s) {
                                tup.push(val.clone());
                            } else {
                                finish = false;
                                break;
                            },
                            ValueId::StackSlot(_) => ()
                        }
                    }

                    if finish {
                        let id = ConstantId(constants.len());
                        let result = IrValue::Tuple(tup);
                        constants.push(Constant { inner: result.clone(), id });
                        self.registers.insert(*d, result);
                        inst.kind = InstKind::Mov(*d, ValueId::Constant(id));
                    }
                },
                InstKind::StoreSS(_, s) => match s {
                    ValueId::Register(rs) => if let Some(val) = self.registers.get(rs) {
                        let id = ConstantId(constants.len());
                        constants.push(Constant { inner: val.clone(), id });
                        *s = ValueId::Constant(id);
                    },
                    _ => (),
                },
                InstKind::Ret(s) => match s {
                    ValueId::Register(r) => if let Some(val) = self.registers.get(&*r).cloned() {
                        let id = ConstantId(constants.len());
                        constants.push(Constant { inner: val.clone(), id });
                        *s = ValueId::Constant(id);
                    },
                    _ => (),
                },
            }
        }
    }
}
