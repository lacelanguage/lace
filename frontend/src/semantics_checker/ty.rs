use std::fmt;
use lace_ir::core::ty::Type as IRType;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int, Float, Bool,
    Unit, Tuple(Vec<Type>),
    Function(Vec<Type>, Box<Type>),
}

impl Type {
    pub fn is_numeric(&self) -> bool {
        matches!(self, Self::Int | Self::Float)
    }

    pub fn to_ir_type(&self) -> IRType {
        match self {
            Self::Int => IRType::Int,
            Self::Float => IRType::Float,
            Self::Bool => IRType::Bool,
            Self::Unit => IRType::Unit,
            Self::Tuple(items) => IRType::Tuple(items.iter().map(|ty| ty.to_ir_type()).collect()),
            Self::Function(p, r) => IRType::Function(p.iter().map(|param| param.to_ir_type()).collect(), Box::new(r.to_ir_type()))
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int => write!(f, "int"),
            Self::Float => write!(f, "float"),
            Self::Bool => write!(f, "bool"),
            Self::Unit => write!(f, "()"),
            Self::Tuple(items) => write!(
                f, "({})",
                items.iter().enumerate().fold(String::new(), |acc, (idx, ty)| {
                    if items.len() == 1 {
                        format!("{ty},")
                    } else if idx > 0 {
                        format!("{acc}, {ty}")
                    } else {
                        ty.to_string()
                    }
                })
            ),
            Self::Function(params, ret) => write!(
                f, "fn({}) -> {ret}",
                params.iter().enumerate().fold(String::new(), |acc, (idx, ty)| {
                    if idx > 0 {
                        format!("{acc}, {ty}")
                    } else {
                        ty.to_string()
                    }
                })
            ),
        }
    }
}