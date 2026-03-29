use crate::semantics_checker::ty::Type;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Op {
    Plus, Minus, Star, Slash, Modulo, Power,
    Bang, Assign,
    Eq, Ne, Gt, Ge, Lt, Le,
}

impl Op {
    pub fn binding_power(&self) -> (usize, usize) {
        match self {
            Self::Assign => (10, 11),
            Self::Eq | Self::Ne => (20, 21),
            Self::Gt | Self::Ge | Self::Lt | Self::Le => (30, 31),
            Self::Plus | Self::Minus => (40, 41),
            Self::Star | Self::Slash | Self::Modulo => (50, 51),
            Self::Power => (61, 60),
            _ => (0, 0), // prefix ops are handled differently
        }
    }

    pub fn is_infix(&self) -> bool {
        ![Self::Bang].contains(self)
    }

    pub fn is_prefix(&self) -> bool {
        [Self::Bang, Self::Plus, Self::Minus].contains(self)
    }

    pub fn infix_output_ty(&self, lhs: &Type, rhs: &Type) -> Option<Type> {
        match self {
            Self::Plus | Self::Minus
            | Self::Star | Self::Slash
            | Self::Modulo | Self::Power => if lhs.is_numeric() && lhs == rhs {
                Some(lhs.clone())
            } else {
                None
            },
            Self::Eq | Self::Ne => if lhs == rhs {
                Some(lhs.clone())
            } else {
                None
            },
            Self::Gt | Self::Ge | Self::Lt | Self::Le => if lhs.is_numeric() && lhs == rhs {
                Some(Type::Bool)
            } else {
                None
            },
            _ => None
        }
    }

    pub fn prefix_output_ty(&self, operand: &Type) -> Option<Type> {
        match self {
            Self::Plus | Self::Minus => if operand.is_numeric() {
                Some(operand.clone())
            } else {
                None
            },
            Self::Bang => if matches!(operand, Type::Int) {
                Some(Type::Int)
            } else {
                None
            },
            _ => None
        }
    }
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Star => "*",
            Self::Slash => "/",
            Self::Modulo => "%",
            Self::Power => "**",
            Self::Eq => "==",
            Self::Ne => "!=",
            Self::Gt => ">",
            Self::Ge => ">=",
            Self::Lt => "<",
            Self::Le => "<=",
            Self::Bang => "!",
            Self::Assign => "=",
        })
    }
}