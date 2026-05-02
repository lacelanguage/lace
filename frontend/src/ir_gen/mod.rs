use std::collections::HashMap;
use lace_ir::core::module::Module;
use lace_ir::core::function::{FunctionName, Signature};
use lace_ir::core::inst::{IrValue, Register, ValueId, CmpFlag};
use lace_ir::core::ss::SlotId;
use lasso::Spur;
use crate::operator::Op;
use crate::parser::ast::{Ast, FuncId, Node, NodeKind, RootLevelItem, RootLevelItemKind};
use crate::semantics_checker::ty::Type;
use crate::semantics_checker::type_map::{FunctionDefTypeInfo, TypeMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Symbol {
    Variable(SlotId),
    Arg(Register),
}

pub struct IRGenerator {
    pub module: Module,
    pub functions: HashMap<FuncId, FunctionName>,
    pub scope: Vec<HashMap<Spur, Symbol>>,
}

impl IRGenerator {
    pub fn new<S: AsRef<str>>(id: usize, name: S) -> Self {
        Self {
            module: Module::new(id, name),
            functions: HashMap::new(),
            scope: vec![HashMap::new()]
        }
    }

    pub fn generate_ir(&mut self, ast: &Ast, type_map: &TypeMap) {
        for item in ast.0.iter() {
            self.collect_root_level_item(item, type_map);
        }

        for item in ast.0.iter() {
            self.walk_root_level_item(item, type_map);
        }
    }

    pub fn collect_root_level_item(&mut self, item: &RootLevelItem, type_map: &TypeMap) {
        match &item.kind {
            RootLevelItemKind::FunctionDef(f) => {
                let FunctionDefTypeInfo { params, return_ty, .. } = type_map.get_func(f.id).unwrap();
                let sig = Signature {
                    params: params.iter().map(|p| p.to_ir_type()).fold(
                        vec![],
                        |mut acc, p| {
                            let reg = Register(acc.len());
                            acc.push((reg, p));
                            acc
                        }
                    ),
                    return_ty: return_ty.to_ir_type()
                };
                self.functions.insert(f.id, self.module.define_function(0, f.name, sig));
            }
        }
    }

    pub fn walk_root_level_item(&mut self, item: &RootLevelItem, type_map: &TypeMap) {
        match &item.kind {
            RootLevelItemKind::FunctionDef(f) => {
                self.scope.push(HashMap::new());
                let fname = *self.functions.get(&f.id).unwrap();
                {
                    let func = self.module.get_function(&fname).unwrap();
                    let entry_block = func.create_block();
                    func.switch_to_block(entry_block);
                    for (idx, (p_ty, p)) in type_map.get_func(f.id).unwrap().params.iter().zip(f.params.iter()).enumerate() {
                        if !p.mutability {
                            self.scope.last_mut().unwrap().insert(p.name, Symbol::Arg(func.get_function_param(idx)));
                            continue;
                        }
                        let ss = func.create_stack_slot(p_ty.to_ir_type());
                        self.scope.last_mut().unwrap().insert(p.name, Symbol::Variable(ss));
                    }
                }
                
                let val = self.walk_node(&f.body, type_map, &fname);
                self.scope.pop();

                self.module.get_function(&fname).unwrap().ib(f.body.span).ret(val);
            }
        }
    }

    pub fn find_ident(&self, n: &Spur) -> Option<&Symbol> {
        for scope in self.scope.iter().rev() {
            if let Some(sy) = scope.get(n) {
                return Some(sy);
            }
        }
        None
    }

    pub fn walk_node(&mut self, node: &Node, type_map: &TypeMap, func: &FunctionName) -> ValueId {
        match &node.kind {
            NodeKind::Identifier(n) => match *self.find_ident(n).unwrap() {
                Symbol::Variable(s) => ValueId::StackSlot(s),
                Symbol::Arg(r) => ValueId::Register(r)
            },
            NodeKind::IntLit(n) => ValueId::Constant(self.module.get_function(func).unwrap().define_constant(IrValue::Int(*n))),
            NodeKind::FloatLit(n) => ValueId::Constant(self.module.get_function(func).unwrap().define_constant(IrValue::Float(*n))),
            NodeKind::Unit => ValueId::Constant(self.module.get_function(func).unwrap().define_constant(IrValue::Unit)),
            NodeKind::Semi(stmt) => {
                self.walk_node(stmt, type_map, func);
                ValueId::Constant(self.module.get_function(func).unwrap().define_constant(IrValue::Unit))
            },
            NodeKind::Tuple(items) => {
                let mut generated_items = vec![];
                for i in items {
                    generated_items.push(self.walk_node(i, type_map, func));
                }
                ValueId::Register(self.module.get_function(func).unwrap().ib(node.span).make_tuple(generated_items))
            },
            NodeKind::Block(stmts) => {
                let mut last_value = ValueId::Constant(self.module.get_function(func).unwrap().define_constant(IrValue::Unit));

                self.scope.push(HashMap::new());

                for stmt in stmts {
                    last_value = self.walk_node(stmt, type_map, func);
                }

                self.scope.pop();

                last_value
            },
            NodeKind::BinaryOp { lhs, rhs, op } => {
                if op.0 == Op::Assign {
                    if let NodeKind::Identifier(n) = &lhs.kind {
                        let val = self.walk_node(rhs, type_map, func);
                        match *self.scope.last().unwrap().get(n).unwrap() {
                            Symbol::Variable(r) => self.module.get_function(func).unwrap().ib(node.span).store_ss(r, val),
                            _ => unreachable!()
                        }
                        return val;
                    }
                }

                let lval = self.walk_node(lhs, type_map, func);
                let rval = self.walk_node(rhs, type_map, func);

                let ib = self.module.get_function(func).unwrap().ib(node.span);
                match op.0 {
                    Op::Assign => unreachable!(),
                    Op::Plus => if *type_map.get_node(lhs.id).unwrap() == Type::Int {
                        ValueId::Register(ib.iadd(lval, rval))
                    } else {
                        ValueId::Register(ib.fadd(lval, rval))
                    },
                    Op::Minus => if *type_map.get_node(lhs.id).unwrap() == Type::Int {
                        ValueId::Register(ib.isub(lval, rval))
                    } else {
                        ValueId::Register(ib.fsub(lval, rval))
                    },
                    Op::Star => if *type_map.get_node(lhs.id).unwrap() == Type::Int {
                        ValueId::Register(ib.imul(lval, rval))
                    } else {
                        ValueId::Register(ib.fmul(lval, rval))
                    },
                    Op::Slash => if *type_map.get_node(lhs.id).unwrap() == Type::Int {
                        ValueId::Register(ib.idiv(lval, rval))
                    } else {
                        ValueId::Register(ib.fdiv(lval, rval))
                    },
                    Op::Modulo => if *type_map.get_node(lhs.id).unwrap() == Type::Int {
                        ValueId::Register(ib.irem(lval, rval))
                    } else {
                        ValueId::Register(ib.frem(lval, rval))
                    },
                    Op::Power => if *type_map.get_node(lhs.id).unwrap() == Type::Int {
                        ValueId::Register(ib.ipow(lval, rval))
                    } else {
                        ValueId::Register(ib.fpow(lval, rval))
                    },
                    Op::Eq => if *type_map.get_node(lhs.id).unwrap() == Type::Int {
                        ValueId::Register(ib.icmp(lval, rval, CmpFlag::Eq))
                    } else {
                        ValueId::Register(ib.fcmp(lval, rval, CmpFlag::Eq))
                    },
                    Op::Ne => if *type_map.get_node(lhs.id).unwrap() == Type::Int {
                        ValueId::Register(ib.icmp(lval, rval, CmpFlag::Ne))
                    } else {
                        ValueId::Register(ib.fcmp(lval, rval, CmpFlag::Ne))
                    },
                    Op::Gt => if *type_map.get_node(lhs.id).unwrap() == Type::Int {
                        ValueId::Register(ib.icmp(lval, rval, CmpFlag::Gt))
                    } else {
                        ValueId::Register(ib.fcmp(lval, rval, CmpFlag::Gt))
                    },
                    Op::Lt => if *type_map.get_node(lhs.id).unwrap() == Type::Int {
                        ValueId::Register(ib.icmp(lval, rval, CmpFlag::Lt))
                    } else {
                        ValueId::Register(ib.fcmp(lval, rval, CmpFlag::Lt))
                    },
                    Op::Ge => if *type_map.get_node(lhs.id).unwrap() == Type::Int {
                        ValueId::Register(ib.icmp(lval, rval, CmpFlag::Ge))
                    } else {
                        ValueId::Register(ib.fcmp(lval, rval, CmpFlag::Ge))
                    },
                    Op::Le => if *type_map.get_node(lhs.id).unwrap() == Type::Int {
                        ValueId::Register(ib.icmp(lval, rval, CmpFlag::Le))
                    } else {
                        ValueId::Register(ib.fcmp(lval, rval, CmpFlag::Le))
                    },
                    _ => todo!(),
                }
            },
            NodeKind::UnaryOp { operand, op } => {
                let oval = self.walk_node(operand, type_map, func);

                let f = self.module.get_function(func).unwrap();
                match op.0 {
                    Op::Plus => oval,
                    Op::Minus => if *type_map.get_node(operand.id).unwrap() == Type::Int {
                        let zero = f.define_constant(IrValue::Int(0));
                        ValueId::Register(f.ib(node.span).isub(ValueId::Constant(zero), oval))
                    } else {
                        let zero = f.define_constant(IrValue::Float(0.0));
                        ValueId::Register(f.ib(node.span).fsub(ValueId::Constant(zero), oval))
                    },
                    Op::Bang => if *type_map.get_node(operand.id).unwrap() == Type::Bool {
                        todo!("logical NOT")
                    } else {
                        todo!("bitwise NOT")
                    },
                    _ => unreachable!()
                }
            },
            NodeKind::Let { mutability: _, name, ty: _, value } => {
                let r = self.walk_node(value, type_map, func);
                let f = self.module.get_function(func).unwrap();
                let ss = f.create_stack_slot(type_map.get_node(node.id).unwrap().to_ir_type());
                f.ib(node.span).store_ss(ss, r);
                self.scope.last_mut().unwrap().insert(*name, Symbol::Variable(ss));
                r
            },
            NodeKind::FunctionDef(_f) => todo!(),
            NodeKind::If { condition, then_body, else_body } => {
                let f = self.module.get_function(func).unwrap();
                let then_b = f.create_block();
                let else_b = f.create_block();
                let merge_b = f.create_block();
                f.append_block_params(merge_b, vec![type_map.get_node(node.id).unwrap().to_ir_type()]);

                let cond_v = self.walk_node(condition, type_map, func);
                let f = self.module.get_function(func).unwrap();
                f.ib(node.span).brif(cond_v, then_b, vec![], else_b, vec![]);

                f.switch_to_block(then_b);
                let then_v = self.walk_node(then_body, type_map, func);
                let f = self.module.get_function(func).unwrap();
                f.ib(then_body.span).jmp(merge_b, vec![then_v]);

                f.switch_to_block(else_b);
                let else_v = match else_body.as_ref().map(|e| self.walk_node(e, type_map, func)) {
                    Some(v) => v,
                    None => ValueId::Constant(self.module.get_function(func).unwrap().define_constant(IrValue::Unit)),
                };
                let f = self.module.get_function(func).unwrap();
                f.ib(then_body.span).jmp(merge_b, vec![else_v]);
                
                f.switch_to_block(merge_b);
                ValueId::Register(f.get_block_param(0))
            },
        }
    }
}