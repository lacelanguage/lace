use super::ty::Type;
use crate::span::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub mutability: bool,
    pub ty: Type,
    pub defined_at: Span,
}