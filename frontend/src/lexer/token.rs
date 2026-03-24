use crate::span::Span;
use crate::operator::Op;
use crate::diagnostic::{Diagnostic, Severity};
use lasso::{Spur, Rodeo};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenKind {
    Identifier(Spur),
    IntLit(i64),
    FloatLit(f64),
    Operator(Op),
    LParen, RParen,
}

impl TokenKind {
    pub fn as_str(&self, rodeo: &Rodeo) -> String {
        match self {
            Self::Identifier(spur) => rodeo.resolve(spur).to_string(),
            Self::IntLit(i) => i.to_string(),
            Self::FloatLit(i) => i.to_string(),
            Self::Operator(op) => op.as_str().to_string(),
            Self::LParen => "(".to_string(),
            Self::RParen => ")".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenStream {
    pub tokens: Arc<[Token]>,
    pub pos: usize,
}

impl TokenStream {
    pub fn consume(&mut self, expected: TokenKind, rodeo: &Rodeo) -> Result<Token, Diagnostic> {
        match self.tokens.get(self.pos) {
            Some(tok) => if tok.kind == expected {
                Ok(*tok)
            } else {
                Err(Diagnostic::new(
                    Severity::Error,
                    format!(
                        "expected `{}`, found `{}`",
                        expected.as_str(rodeo),
                        tok.kind.as_str(rodeo)
                    ),
                    tok.span
                ))
            },
            None => Err(Diagnostic::new(
                Severity::Error,
                format!(
                    "expected `{}`, found end of input",
                    expected.as_str(rodeo)
                ),
                self.tokens.last()
                    .map(|tok| tok.span)
                    .unwrap_or(Span::empty())
            ))
        }
    }

    pub fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    pub fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }
}