use std::fmt;
use super::ss::{StackSlot, SlotId};
use super::basic_block::{Block, BlockId};
use super::inst::*;
use super::ty::Type;
use lace_span::Span;
use lasso::Spur;

#[derive(Clone, PartialEq)]
pub struct Constant {
    pub inner: IrValue,
    pub id: ConstantId,
}

impl fmt::Debug for Constant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {:?}", self.id, self.inner)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionName(pub usize, pub usize, pub usize);

impl fmt::Debug for FunctionName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.0, self.1, self.2)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Signature {
    pub params: Vec<(Register, Type)>,
    pub return_ty: Type,
}

#[derive(Clone, PartialEq)]
pub struct Function {
    pub constants: Vec<Constant>,
    pub spur: Spur,
    pub name: FunctionName,
    pub sig: Signature,
    pub blocks: Vec<Block>,
    pub current_block: Option<BlockId>,
    pub next_value_id: usize,
    pub stack_slots: Vec<StackSlot>
}

impl Function {
    pub fn new(name: Spur, id: usize, namespace: usize, scope: usize, sig: Signature) -> Self {
        let next_value_id = sig.params.len();
        Self {
            constants: vec![],
            spur: name,
            name: FunctionName(namespace, scope, id), sig,
            blocks: vec![], current_block: None,
            next_value_id,
            stack_slots: vec![]
        }
    }

    pub fn get_block_param(&self, idx: usize) -> Register {
        if let Some(block) = self.current_block {
            self.blocks[block.0].params[idx].0
        } else {
            panic!("No block selected");
        }
    }

    pub fn get_function_param(&self, idx: usize) -> Register {
        Register(idx)
    }

    pub fn create_stack_slot(&mut self, slot_ty: Type) -> SlotId {
        let id = SlotId(self.stack_slots.len());
        self.stack_slots.push(StackSlot { id, slot_ty });
        id
    }

    pub fn allocate_register(&mut self) -> Register {
        let reg = Register(self.next_value_id);
        self.next_value_id += 1;
        reg
    }

    pub fn define_constant(&mut self, v: IrValue) -> ConstantId {
        if let Some(i) = self.constants.iter().find(|c| c.inner == v) {
            return i.id;
        }

        let id = ConstantId(self.constants.len());
        self.constants.push(Constant { inner: v, id });
        id
    }

    pub fn create_block(&mut self) -> BlockId {
        let id = BlockId(self.blocks.len());

        self.blocks.push(Block { id, params: vec![], insts: vec![] });

        id
    }

    pub fn append_block_params(&mut self, id: BlockId, params: Vec<Type>) {
        self.blocks[id.0].params = params.into_iter().map(|ty| {
            (self.allocate_register(), ty)
        }).collect();
    }

    pub fn switch_to_block(&mut self, id: BlockId) {
        self.current_block = Some(id);
    }

    pub fn ib(&mut self, span: Span) -> InstBuilder<'_> {
        InstBuilder::new(self, span)
    }

    pub fn debug_sig(&self) -> String {
        format!("fn {:?}({}) -> {:?}",
            self.name,
            self.sig.params.iter()
                .fold(
                    String::new(),
                    |acc, (p, ty)| if acc.is_empty() {
                        format!("{p:?}: {ty:?}")
                    } else {
                        format!("{acc}, {p:?}: {ty:?}")
                    }
                ),
            self.sig.return_ty
        )
    }

    pub fn debug(&self, rodeo: &lasso::Rodeo) -> String {
        let mut output = String::new();

        output.push_str(&format!("@fn_name({})\n", rodeo.resolve(&self.spur)));
        output.push_str(&format!("{self:?}"));

        output
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "fn {:?}({}) -> {:?} {{",
            self.name,
            self.sig.params.iter()
                .fold(
                    String::new(),
                    |acc, (p, ty)| if acc.is_empty() {
                        format!("{p:?}: {ty:?}")
                    } else {
                        format!("{acc}, {p:?}: {ty:?}")
                    }
                ),
            self.sig.return_ty
        )?;

        if !self.constants.is_empty() {
            writeln!(f, 
                ".CONSTANTS: [{}]",
                self.constants.iter()
                    .fold(String::new(),
                        |acc, c| if acc.is_empty() {
                            format!("{:?}", c)
                        } else {
                            format!("{acc}, {:?}", c)
                        }
                    )
            )?;
        }

        if !self.stack_slots.is_empty() {
            writeln!(f, 
                ".STACK_SLOTS: [{}]",
                self.stack_slots.iter()
                    .fold(String::new(),
                        |acc, ss| if acc.is_empty() {
                            format!("{:?}", ss)
                        } else {
                            format!("{acc}, {:?}", ss)
                        }
                    )
            )?;
        }

        for block in &self.blocks {
            writeln!(f, "{block:?}")?;
        }

        write!(f, "}}")
    }
}

#[cfg(test)]
pub mod tests {
    use lace_span::Span;
    use super::{Function, Signature};
    use super::super::inst::*;
    use super::super::ty::Type;

    #[test]
    fn format_test() {
        let mut rodeo = lasso::Rodeo::new();

        let sig = Signature {
            params: vec![(Register(0), Type::Int), (Register(1), Type::Int)],
            return_ty: Type::Int,
        };

        let mut function = Function::new(rodeo.get_or_intern("add"), 0, 0, 0, sig);
        let entry = function.create_block();
        function.switch_to_block(entry);
        function.ib(Span::empty()).iadd(ValueId::Register(Register(0)), ValueId::Register(Register(1)));
        function.ib(Span::empty()).ret(ValueId::Register(Register(2)));

        println!("{}", function.debug(&rodeo));
    }
}