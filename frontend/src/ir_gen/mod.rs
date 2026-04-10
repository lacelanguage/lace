use std::collections::HashMap;
use lace_ir::module::Module;
use lace_ir::function::{FunctionName, Signature};
use lace_ir::inst::{IrValue, Register, ValueId};
use lasso::Spur;
use crate::operator::Op;
use crate::parser::ast::{Ast, FuncId, Node, NodeKind, RootLevelItem, RootLevelItemKind};
use crate::semantics_checker::ty::Type;
use crate::semantics_checker::type_map::{FunctionDefTypeInfo, TypeMap};

pub struct IRGenerator {
    pub module: Module,
    pub functions: HashMap<FuncId, FunctionName>,
    pub scope: Vec<HashMap<Spur, Register>>,
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
                let fname = self.functions.get(&f.id).unwrap().clone();
                {
                    let f = self.module.get_function(&fname).unwrap();
                    let entry_block = f.create_block();
                    f.switch_to_block(entry_block);
                }
                
                let val = self.walk_node(&f.body, type_map, &fname);

                self.module.get_function(&fname).unwrap().ib(f.body.span).ret(val);
            }
        }
    }

    pub fn walk_node(&mut self, node: &Node, type_map: &TypeMap, func: &FunctionName) -> ValueId {
        match &node.kind {
            NodeKind::Identifier(n) => ValueId::Register(*self.scope.last().unwrap().get(n).unwrap()),
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
                    if let NodeKind::Identifier(n) = lhs.kind {
                        let val = self.walk_node(rhs, type_map, func);
                        let new_r = self.module.get_function(func).unwrap().ib(node.span).mov(val);
                        for s in self.scope.iter_mut().rev() {
                            if let Some(r) = s.get_mut(&n) {
                                *r = new_r;
                                break;
                            }
                        }
                        return ValueId::Constant(self.module.get_function(func).unwrap().define_constant(IrValue::Unit));
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
                let reg = f.ib(node.span).mov(r);
                self.scope.last_mut().unwrap().insert(*name, reg);
                ValueId::Constant(f.define_constant(IrValue::Unit))
            },
            NodeKind::FunctionDef(_f) => todo!()
        }
    }
}