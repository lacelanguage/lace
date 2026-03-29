use crate::span::Span;
use lasso::Spur;

#[derive(Debug, Clone, PartialEq)]
pub enum ParseTypeKind {
    Identifier(Spur),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseType {
    pub kind: ParseTypeKind,
    pub span: Span
}