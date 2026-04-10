use super::ty::Type;
use lace_span::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub mutability: bool,
    pub ty: Type,
    pub defined_at: Span,
}