use std::fmt;
use super::basic_block::{Block, BlockId};
use super::inst::*;
use super::ty::Type;
use lace_span::Span;
use lace_vm::value::ConstantId;
use lasso::Spur;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionName(pub usize, pub usize, pub Spur);

impl fmt::Debug for FunctionName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.0, self.1, self.2.into_inner())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Signature {
    pub params: Vec<(Register, Type)>,
    pub return_ty: Type,
}

#[derive(Clone, PartialEq)]
pub struct Function {
    pub constants: Vec<IrValue>,
    pub name: FunctionName,
    pub sig: Signature,
    pub blocks: Vec<Block>,
    pub current_block: Option<BlockId>,
    pub next_value_id: usize,
}

impl Function {
    pub fn new(name: Spur, namespace: usize, scope: usize, sig: Signature) -> Self {
        let next_value_id = sig.params.len();
        Self {
            constants: vec![],
            name: FunctionName(namespace, scope, name), sig,
            blocks: vec![], current_block: None,
            next_value_id
        }
    }

    pub fn allocate_register(&mut self) -> Register {
        let reg = Register(self.next_value_id);
        self.next_value_id += 1;
        reg
    }

    pub fn define_constant(&mut self, v: IrValue) -> ConstantId {
        if let Some(v) = self.constants.iter().position(|c| *c == v) {
            return ConstantId(v);
        }

        let id = ConstantId(self.constants.len());
        self.constants.push(v);
        id
    }

    pub fn create_block(&mut self) -> BlockId {
        let id = BlockId(self.blocks.len());

        self.blocks.push(Block { id, insts: vec![] });

        id
    }

    pub fn switch_to_block(&mut self, id: BlockId) {
        self.current_block = Some(id);
    }

    pub fn ib(&mut self, span: Span) -> InstBuilder<'_> {
        InstBuilder::new(self, span)
    }

    pub fn debug(&self, rodeo: &lasso::Rodeo) -> String {
        let mut output = String::new();

        output.push_str(&format!("@fn_name({})\n", rodeo.resolve(&self.name.2)));
        output.push_str(&format!("{self:?}"));

        output
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "fn {:?}({}) -> {:?} {{\n",
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

        write!(f, 
            ".CONSTANTS: [{}]\n",
            self.constants.iter()
                .fold(String::new(),
                    |acc, c| if acc.is_empty() {
                        format!("{:?}", c)
                    } else {
                        format!("{acc}, {:?}", c)
                    }
                )
        )?;

        for block in &self.blocks {
            write!(f, "{block:?}\n")?;
        }

        write!(f, "}}")
    }
}

#[cfg(test)]
pub mod tests {
    use lace_span::Span;
    use crate::function::{Function, Signature};
    use crate::inst::*;
    use crate::ty::Type;

    #[test]
    fn format_test() {
        let mut rodeo = lasso::Rodeo::new();

        let sig = Signature {
            params: vec![(Register(0), Type::Int), (Register(1), Type::Int)],
            return_ty: Type::Int,
        };

        let mut function = Function::new(rodeo.get_or_intern("add"), 0, 0, sig);
        let entry = function.create_block();
        function.switch_to_block(entry);
        function.ib(Span::empty()).iadd(ValueId::Register(Register(0)), ValueId::Register(Register(1)));
        function.ib(Span::empty()).ret(ValueId::Register(Register(2)));

        println!("{}", function.debug(&rodeo));
    }
}