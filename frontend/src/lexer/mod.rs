pub mod token;

use token::*;
use lace_span::Span;
use crate::operator::Op;
use crate::diagnostic::{Diagnostic, Severity};
use lasso::Rodeo;

pub fn tokenize(source: &str, rodeo: &mut Rodeo) -> Result<TokenStream, Diagnostic> {
    let mut tokens = Vec::new();
    let mut chars = source.char_indices().peekable();
    
    while let Some((start, ch)) = chars.next() {
        match ch {
            ' ' | '\t' | '\r' | '\n' => continue,
            ':' => tokens.push(Token {
                kind: TokenKind::Colon,
                span: Span {
                    start,
                    end: start + ch.len_utf8()
                },
            }),
            ',' => tokens.push(Token {
                kind: TokenKind::Comma,
                span: Span {
                    start,
                    end: start + ch.len_utf8()
                },
            }),
            ';' => tokens.push(Token {
                kind: TokenKind::Semicolon,
                span: Span {
                    start,
                    end: start + ch.len_utf8()
                },
            }),
            '+' => tokens.push(Token {
                kind: TokenKind::Operator(Op::Plus),
                span: Span {
                    start,
                    end: start + ch.len_utf8()
                },
            }),
            '-' => if let Some(&(end, '>')) = chars.peek() {
                chars.next();
                tokens.push(Token {
                    kind: TokenKind::Arrow,
                    span: Span {
                        start,
                        end: end + '>'.len_utf8()
                    },
                });
            } else {
                tokens.push(Token {
                    kind: TokenKind::Operator(Op::Minus),
                    span: Span {
                        start,
                        end: start + ch.len_utf8()
                    },
                });
            },
            '*' => if let Some(&(end, '*')) = chars.peek() {
                chars.next();

                tokens.push(Token {
                    kind: TokenKind::Operator(Op::Power),
                    span: Span {
                        start,
                        end: end + '*'.len_utf8()
                    },
                });
            } else {
                tokens.push(Token {
                    kind: TokenKind::Operator(Op::Star),
                    span: Span {
                        start,
                        end: start + ch.len_utf8()
                    },
                });
            },
            '/' => tokens.push(Token {
                kind: TokenKind::Operator(Op::Slash),
                span: Span {
                    start,
                    end: start + ch.len_utf8()
                },
            }),
            '%' => tokens.push(Token {
                kind: TokenKind::Operator(Op::Modulo),
                span: Span {
                    start,
                    end: start + ch.len_utf8()
                },
            }),
            '(' => tokens.push(Token {
                kind: TokenKind::LParen,
                span: Span {
                    start,
                    end: start + ch.len_utf8()
                },
            }),
            ')' => tokens.push(Token {
                kind: TokenKind::RParen,
                span: Span {
                    start,
                    end: start + ch.len_utf8()
                },
            }),
            '{' => tokens.push(Token {
                kind: TokenKind::LCurly,
                span: Span {
                    start,
                    end: start + ch.len_utf8()
                },
            }),
            '}' => tokens.push(Token {
                kind: TokenKind::RCurly,
                span: Span {
                    start,
                    end: start + ch.len_utf8()
                },
            }),
            '=' => if let Some(&(end, '=')) = chars.peek() {
                chars.next();
                tokens.push(Token {
                    kind: TokenKind::Operator(Op::Eq),
                    span: Span {
                        start,
                        end: end + '='.len_utf8()
                    },
                });
            } else {
                tokens.push(Token {
                    kind: TokenKind::Operator(Op::Assign),
                    span: Span {
                        start,
                        end: start + '='.len_utf8()
                    },
                });
            },
            '>' => if let Some(&(end, '=')) = chars.peek() {
                chars.next();
                tokens.push(Token {
                    kind: TokenKind::Operator(Op::Ge),
                    span: Span {
                        start,
                        end: end + '='.len_utf8()
                    },
                });
            } else {
                tokens.push(Token {
                    kind: TokenKind::Operator(Op::Gt),
                    span: Span {
                        start,
                        end: start + '='.len_utf8()
                    },
                });
            },
            '<' => if let Some(&(end, '=')) = chars.peek() {
                chars.next();
                tokens.push(Token {
                    kind: TokenKind::Operator(Op::Le),
                    span: Span {
                        start,
                        end: end + '='.len_utf8()
                    },
                });
            } else {
                tokens.push(Token {
                    kind: TokenKind::Operator(Op::Lt),
                    span: Span {
                        start,
                        end: start + '='.len_utf8()
                    },
                });
            },
            '!' => if let Some(&(end, '=')) = chars.peek() {
                chars.next();
                tokens.push(Token {
                    kind: TokenKind::Operator(Op::Ne),
                    span: Span {
                        start,
                        end: end + '='.len_utf8()
                    },
                });
            } else {
                tokens.push(Token {
                    kind: TokenKind::Operator(Op::Bang),
                    span: Span {
                        start,
                        end: start + '='.len_utf8()
                    },
                });
            },
            '0'..='9' => {
                let mut is_float = false;
                let mut end = start;
                let mut last_ch_len = ch.len_utf8();

                while let Some(&(pos, ch)) = chars.peek() {
                    match ch {
                        '0'..='9' | '_' => chars.next(),
                        '.' => if is_float {
                            break;
                        } else {
                            is_float = true;
                            chars.next()
                        },
                        _ => break,
                    };

                    end = pos;
                    last_ch_len = ch.len_utf8();
                }

                end += last_ch_len;

                if is_float {
                    tokens.push(Token {
                        kind: TokenKind::FloatLit(source[start..end].replace('_', "").parse().unwrap()),
                        span: Span { start, end }
                    });
                } else {
                    tokens.push(Token {
                        kind: TokenKind::IntLit(source[start..end].replace('_', "").parse().unwrap()),
                        span: Span { start, end }
                    });
                }
            },
            ch if ch.is_alphabetic() || ch == '_' => {
                let mut end = start;
                let mut last_ch_len = ch.len_utf8();

                while let Some(&(pos, ch)) = chars.peek() {
                    match ch {
                        ch if ch.is_alphabetic() || ch == '_' || ch.is_ascii_digit() => chars.next(),
                        _ => break,
                    };

                    end = pos;
                    last_ch_len = ch.len_utf8();
                }

                end += last_ch_len;

                tokens.push(Token {
                    kind: lookup_ident(&source[start..end], rodeo),
                    span: Span { start, end}
                });
            },
            ch => return Err(Diagnostic::new(
                Severity::Error,
                format!("unrecognized character: `{ch}`"), 
                Span {
                    start,
                    end: start + ch.len_utf8()
                }
            )),
        }
    }

    Ok(TokenStream {
        tokens: tokens.into(),
        pos: 0usize
    })
}

pub fn lookup_ident(source: &str, rodeo: &mut Rodeo) -> TokenKind {
    match source {
        "let" => TokenKind::KwLet,
        "mut" => TokenKind::KwMut,
        "fn" => TokenKind::KwFn,
        _ => TokenKind::Identifier(rodeo.get_or_intern(source)),
    }
}