use std::fmt;

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