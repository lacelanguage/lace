use super::ty::ParseType;
use lace_span::Span;
use crate::operator::Op;
use lasso::Spur;
use std::sync::Arc;

// usize instead of u32 for simplicity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FuncId(pub usize);

#[derive(Debug, Clone, PartialEq)]
pub struct Ast(pub Arc<[RootLevelItem]>);

#[derive(Debug, Clone, PartialEq)]
pub enum RootLevelItemKind {
    FunctionDef(FunctionDef),
    // later: Const, Import, etc.
}

#[derive(Debug, Clone, PartialEq)]
pub struct RootLevelItem {
    pub kind: RootLevelItemKind,
    pub span: Span
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub id: NodeId,
    pub kind: NodeKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    Identifier(Spur),
    IntLit(i64),
    FloatLit(f64),
    Unit,
    Semi(Box<Node>),
    Tuple(Vec<Node>),
    Block(Vec<Node>),
    BinaryOp {
        lhs: Box<Node>,
        rhs: Box<Node>,
        op: (Op, Span),
    },
    UnaryOp {
        operand: Box<Node>,
        op: (Op, Span),
    },
    Let {
        mutability: bool,
        name: Spur,
        ty: Option<ParseType>,
        value: Box<Node>,
    },
    FunctionDef(FunctionDef),
    If {
        condition: Box<Node>,
        then_body: Box<Node>,
        else_body: Option<Box<Node>>,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDef {
    pub id: FuncId,
    pub name: Spur,
    pub params: Vec<ParseParam>,
    pub return_ty: Option<ParseType>,
    pub body: Box<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseParam {
    pub mutability: bool,
    pub name: Spur,
    pub ty: ParseType,
    pub span: Span,
}