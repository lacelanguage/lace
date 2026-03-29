use crate::span::Span;
use lasso::Spur;

#[derive(Debug, Clone, PartialEq)]
pub enum PatternKind {
    Identifier(Spur),
    Or(Box<Pattern>, Box<Pattern>)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pattern {
    pub kind: PatternKind,
    pub span: Span
}