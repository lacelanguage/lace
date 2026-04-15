use std::fmt;

#[derive(Clone, PartialEq)]
pub enum Type {
    Int, Float, Bool, Unit,
    Tuple(Vec<Type>),
    Function(Vec<Type>, Box<Type>),
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int => write!(f, "int"),
            Self::Float => write!(f, "float"),
            Self::Bool => write!(f, "bool"),
            Self::Unit => write!(f, "unit"),
            Self::Tuple(items) => write!(f, "tup({})",
                items.iter()
                    .fold(
                        String::new(),
                        |acc, t| if acc.is_empty() {
                            format!("{t:?}")
                        } else {
                            format!("{acc}, {t:?}")
                        }
            )),
            Self::Function(params, ret) => write!(f, "fn({}) -> {ret:?}",
                params.iter()
                    .fold(
                        String::new(),
                        |acc, t| if acc.is_empty() {
                            format!("{t:?}")
                        } else {
                            format!("{acc}, {t:?}")
                        }
            )),
        }
    }
}