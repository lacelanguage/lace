use crate::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Secondary {
    Note(String),
    Help(String)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub main_msg: String,
    pub main_span: Span,
    pub secondary: Vec<(Secondary, Option<Span>)>,
}

impl Diagnostic {
    pub fn new<T: AsRef<str>>(severity: Severity, main_msg: T, main_span: Span) -> Self {
        Self {
            severity, main_msg: main_msg.as_ref().to_string(), main_span,
            secondary: vec![]
        }
    }

    pub fn with_note<T: AsRef<str>>(mut self, msg: T, span: Option<Span>) -> Self {
        self.secondary.push((Secondary::Note(msg.as_ref().to_string()), span));
        self
    }

    pub fn with_help<T: AsRef<str>>(mut self, msg: T, span: Option<Span>) -> Self {
        self.secondary.push((Secondary::Help(msg.as_ref().to_string()), span));
        self
    }

    // todo: add real formatting
}