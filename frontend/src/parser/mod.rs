pub mod ast;
pub mod ty;
pub mod pattern;

use ast::*;
use ty::*;
use pattern::*;
use crate::lexer::token::*;
use lace_span::Span;
use crate::operator::Op;
use crate::diagnostic::{Diagnostic, Severity};
use lasso::Rodeo;

pub struct Parser<'a> {
    next_node_id: usize,
    next_func_id: usize,
    token_stream: TokenStream,
    rodeo: &'a Rodeo
}

impl<'a> Parser<'a> {
    pub fn new(token_stream: TokenStream, rodeo: &'a Rodeo) -> Self {
        Self { next_node_id: 0usize, next_func_id: 0usize, token_stream, rodeo }
    }
    
    pub fn new_node(&mut self, kind: NodeKind, span: Span) -> Node {
        let id = NodeId(self.next_node_id);
        self.next_node_id += 1;
        Node { id, kind, span }
    }

    pub fn parse(&mut self) -> Result<Ast, Diagnostic> {
        let mut items = Vec::new();

        while !self.token_stream.is_at_end() {
            items.push(self.parse_root_level_item()?);
        }

        Ok(Ast(items.into()))
    }

    pub fn parse_root_level_item(&mut self) -> Result<RootLevelItem, Diagnostic> {
        match self.token_stream.peek() {
            Some(Token { kind: TokenKind::KwFn, span: _ }) => {
                let (f, span) = self.parse_function_def()?;
                Ok(RootLevelItem {
                    kind: RootLevelItemKind::FunctionDef(f),
                    span
                })
            },
            Some(other) => Err(Diagnostic::new(
                Severity::Error,
                format!("expected root-level item, found `{}`", other.kind.as_str(self.rodeo)),
                other.span,
            )),
            None => Err(Diagnostic::new(
                Severity::Error,
                "expected root-level item, found end of input",
                self.token_stream.get_last_span(),
            ))
        }
    }

    pub fn parse_statement(&mut self) -> Result<Node, Diagnostic> {
        let mut node = self.parse_expression(0)?;
        if let Ok(tok) = self.token_stream.consume(TokenKind::Semicolon, self.rodeo) {
            let mut s = node.span;
            s.extend(tok.span);
            node = self.new_node(NodeKind::Semi(Box::new(node.clone())), s);
        }

        Ok(node)
    }

    pub fn parse_function_def(&mut self) -> Result<(FunctionDef, Span), Diagnostic> {
        let mut whole_span = self.token_stream.consume(TokenKind::KwFn, self.rodeo)?.span;

        let name = self.token_stream.consume_ident(self.rodeo)?.0;

        self.token_stream.consume(TokenKind::LParen, self.rodeo)?;
        let params = self.parse_separator(TokenKind::RParen, TokenKind::Comma, Self::parse_param)?;
        let return_ty = match self.token_stream.consume(TokenKind::Arrow, self.rodeo) {
            Ok(_) => Some(self.parse_type()?),
            Err(_) => None,
        };
        self.token_stream.consume(TokenKind::Operator(Op::Assign), self.rodeo)?;

        let body = self.parse_expression(0)?;
        whole_span.extend(body.span);

        let id = FuncId(self.next_func_id);
        self.next_func_id += 1;
        Ok((FunctionDef { id, name, params, return_ty, body: Box::new(body) }, whole_span))
    }

    pub fn parse_param(&mut self) -> Result<ParseParam, Diagnostic> {
        let (mutability, mut span): (bool, Option<Span>) = match self.token_stream.consume(TokenKind::KwMut, self.rodeo) {
            Ok(tok) => (true, Some(tok.span)),
            Err(_) => (false, None)
        };
        let (name, name_span) = self.token_stream.consume_ident(self.rodeo)?;
        if let Some(s) = span.as_mut() {
            s.extend(name_span);
        } else {
            span = Some(name_span);
        }
        self.token_stream.consume(TokenKind::Colon, self.rodeo)?;
        let ty = self.parse_type()?;
        span.as_mut().unwrap().extend(ty.span);
        Ok(ParseParam { mutability, name, ty, span: span.unwrap() })
    }

    pub fn parse_type(&mut self) -> Result<ParseType, Diagnostic> {
        let token = match self.token_stream.peek() {
            Some(tok) => *tok,
            None => return Err(Diagnostic::new(
                Severity::Error,
                "expected type, found end of input",
                self.token_stream.get_last_span()
            ))
        };

        match token.kind {
            TokenKind::Identifier(spur) => {
                self.token_stream.advance();
                Ok(ParseType {
                    kind: ParseTypeKind::Identifier(spur),
                    span: token.span,
                })
            },
            other => Err(Diagnostic::new(
                Severity::Error,
                format!("expected type, found `{}`", other.as_str(self.rodeo)),
                token.span
            ))
        }
    }

    pub fn parse_separator<T>(&mut self, terminator: TokenKind, sep: TokenKind, f: fn(&mut Self) -> Result<T, Diagnostic>) -> Result<Vec<T>, Diagnostic> {
        let mut items = Vec::new();

        while let Some(tok) = self.token_stream.peek() {
            if tok.kind == terminator {
                break;
            }

            items.push(f(self)?);

            if self.token_stream.consume(sep, self.rodeo).is_err() {
                break;
            }
        }

        self.token_stream.consume(terminator, self.rodeo)?;

        Ok(items)
    }

    pub fn parse_expression(&mut self, min_bp: usize) -> Result<Node, Diagnostic> {
        let mut lhs = self.parse_primary()?;

        while let Some(&Token { kind: TokenKind::Operator(op), span }) = self.token_stream.peek() {
            let bp = if op.is_infix() {
                let (lbp, rbp) = op.binding_power();
                if lbp < min_bp {
                    break;
                }

                rbp
            } else {
                break;
            };
            self.token_stream.advance();

            let rhs = self.parse_expression(bp)?;

            lhs.span.extend(rhs.span);
            lhs = self.new_node(
                NodeKind::BinaryOp {
                    lhs: Box::new(lhs.clone()),
                    rhs: Box::new(rhs),
                    op: (op, span)
                },
                lhs.span
            );
        }

        Ok(lhs)
    }

    pub fn parse_primary(&mut self) -> Result<Node, Diagnostic> {
        let token = match self.token_stream.peek() {
            Some(tok) => *tok,
            None => return Err(Diagnostic::new(
                Severity::Error,
                "expected expression, found end of input",
                self.token_stream.get_last_span()
            ))
        };

        match token.kind {
            TokenKind::Identifier(spur) => {
                self.token_stream.advance();
                Ok(self.new_node(
                    NodeKind::Identifier(spur),
                    token.span,
                ))
            },
            TokenKind::IntLit(i) => {
                self.token_stream.advance();
                Ok(self.new_node(
                    NodeKind::IntLit(i),
                    token.span,
                ))
            },
            TokenKind::FloatLit(f) => {
                self.token_stream.advance();
                Ok(self.new_node(
                    NodeKind::FloatLit(f),
                    token.span,
                ))
            },
            TokenKind::Operator(op) if op.is_prefix() => {
                self.token_stream.advance();
                let operand = self.parse_primary()?;
                let mut span = token.span;
                span.extend(operand.span);
                Ok(self.new_node(
                    NodeKind::UnaryOp { operand: Box::new(operand), op: (op, token.span) },
                    span
                ))
            },
            TokenKind::LParen => self.parse_paren(),
            TokenKind::LCurly => self.parse_block(),
            TokenKind::KwLet => self.parse_let(),
            TokenKind::KwIf => self.parse_if(),
            other => Err(Diagnostic::new(
                Severity::Error,
                format!("expected expression, found `{}`", other.as_str(self.rodeo)),
                token.span
            ))
        }
    }

    pub fn parse_paren(&mut self) -> Result<Node, Diagnostic> {
        let mut span = self.token_stream.consume(TokenKind::LParen, self.rodeo)?.span;

        let mut is_tuple = false;
        let mut items = Vec::new();

        while let Some(tok) = self.token_stream.peek() {
            if tok.kind == TokenKind::RParen {
                break;
            }

            let mut item = self.parse_expression(0)?;
            
            if self.token_stream.consume(TokenKind::Comma, self.rodeo).is_ok() {
                is_tuple = true;
                items.push(item);
            } else {
                if items.is_empty() {
                    item.span.start -= '('.len_utf8();
                    item.span.end += ')'.len_utf8();
                }

                items.push(item);
                break;
            }
        }

        span.extend(self.token_stream.consume(TokenKind::RParen, self.rodeo)?.span);

        Ok(self.new_node(
            if is_tuple {
                NodeKind::Tuple(items)
            } else if items.is_empty() {
                NodeKind::Unit
            } else {
                return Ok(items[0].clone());
            },
            span
        ))
    }

    pub fn parse_block(&mut self) -> Result<Node, Diagnostic> {
        let mut span = self.token_stream.consume(TokenKind::LCurly, self.rodeo)?.span;

        let mut stmts = Vec::new();

        while let Some(tok) = self.token_stream.peek() {
            if tok.kind == TokenKind::RCurly {
                break;
            }

            stmts.push(self.parse_statement()?);
        }

        span.extend(self.token_stream.consume(TokenKind::RCurly, self.rodeo)?.span);

        Ok(self.new_node(
            NodeKind::Block(stmts),
            span
        ))
    }

    pub fn parse_let(&mut self) -> Result<Node, Diagnostic> {
        let mut span = self.token_stream.consume(TokenKind::KwLet, self.rodeo)?.span;
        let mutability = self.token_stream.consume(TokenKind::KwMut, self.rodeo).is_ok();

        // this will be replaced with self.parse_pattern() later
        let name = self.token_stream.consume_ident(self.rodeo)?.0;
        let ty = if self.token_stream.consume(TokenKind::Colon, self.rodeo).is_ok() {
            Some(self.parse_type()?)
        } else {
            None
        };

        self.token_stream.consume(TokenKind::Operator(Op::Assign), self.rodeo)?;

        let value = self.parse_expression(0)?;
        span.extend(value.span);

        Ok(self.new_node(
            NodeKind::Let { mutability, name, ty, value: Box::new(value) },
            span
        ))
    }

    pub fn parse_if(&mut self) -> Result<Node, Diagnostic> {
        let mut span = self.token_stream.consume(TokenKind::KwIf, self.rodeo)?.span;
        let condition = Box::new(self.parse_expression(0)?);
        self.token_stream.consume(TokenKind::KwThen, self.rodeo)?;
        let then_body = Box::new(self.parse_expression(0)?);
        span.extend(then_body.span);
        let else_body = self.token_stream.consume(TokenKind::KwElse, self.rodeo)
            .ok()
            .and({ // not map because we need error propagation
                let body = self.parse_expression(0)?;
                span.extend(body.span);
                Some(Box::new(body))
            });
        Ok(self.new_node(NodeKind::If { condition, then_body, else_body }, span))
    }

    // unused for now
    pub fn parse_pattern(&mut self) -> Result<Pattern, Diagnostic> {
        let token = match self.token_stream.peek() {
            Some(tok) => *tok,
            None => return Err(Diagnostic::new(
                Severity::Error,
                "expected type, found end of input",
                self.token_stream.get_last_span()
            ))
        };

        match token.kind {
            TokenKind::Identifier(spur) => {
                self.token_stream.advance();
                Ok(Pattern {
                    kind: PatternKind::Identifier(spur),
                    span: token.span,
                })
            },
            other => Err(Diagnostic::new(
                Severity::Error,
                format!("expected type, found `{}`", other.as_str(self.rodeo)),
                token.span
            ))
        }
    }
}