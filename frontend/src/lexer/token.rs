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
    Colon, Comma, Arrow, Semicolon,
    LParen, RParen,
    LCurly, RCurly,
    KwLet, KwMut, KwFn,
}

impl TokenKind {
    pub fn as_str(&self, rodeo: &Rodeo) -> String {
        match self {
            Self::Identifier(spur) => rodeo.resolve(spur).to_string(),
            Self::IntLit(i) => i.to_string(),
            Self::FloatLit(i) => i.to_string(),
            Self::Operator(op) => op.to_string(),
            Self::Colon => ":".to_string(),
            Self::Comma => ",".to_string(),
            Self::Arrow => "->".to_string(),
            Self::Semicolon => ";".to_string(),
            Self::LParen => "(".to_string(),
            Self::RParen => ")".to_string(),
            Self::LCurly => "{".to_string(),
            Self::RCurly => "}".to_string(),
            Self::KwLet => "let".to_string(),
            Self::KwMut => "mut".to_string(),
            Self::KwFn => "fn".to_string(),
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
    pub(crate) pos: usize,
}

impl TokenStream {
    pub fn consume(&mut self, expected: TokenKind, rodeo: &Rodeo) -> Result<Token, Diagnostic> {
        match self.tokens.get(self.pos) {
            Some(&tok) => if tok.kind == expected {
                self.advance();
                Ok(tok)
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
                self.get_last_span()
            ))
        }
    }

    pub fn consume_ident(&mut self, rodeo: &Rodeo) -> Result<(Spur, Span), Diagnostic> {
        match self.tokens.get(self.pos) {
            Some(&tok) => match tok.kind {
                TokenKind::Identifier(spur) => {
                    self.advance();
                    Ok((spur, tok.span))
                },
                other => Err(Diagnostic::new(
                    Severity::Error,
                    format!(
                        "expected identifier, found `{}`",
                        other.as_str(rodeo)
                    ),
                    tok.span
                ))
            },
            None => Err(Diagnostic::new(
                Severity::Error,
                "expected identifier, found end of input",
                self.get_last_span()
            ))
        }
    }

    pub fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    pub fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    pub fn get_last_span(&self) -> Span {
        self.tokens.last()
            .map(|tok| tok.span)
            .unwrap_or(Span::empty())
    }

    pub fn advance(&mut self) {
        self.pos += 1;
    }
}