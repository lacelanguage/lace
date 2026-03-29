use tinycolor::Colorize;
use crate::span::Span;
use crate::utils;

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

    pub fn display(&self, path: &str, lines: &[&str], line_starts: &[usize]) -> String {
        let mut output = String::new();

        let start_line = utils::line_of(self.main_span.start, line_starts);
        let start_col = self.main_span.start - line_starts[start_line];
        let end_line = utils::line_of(self.main_span.end, line_starts);
        let end_col = self.main_span.end - line_starts[end_line];

        let digit_len = (end_col + 1).ilog10() as usize + 1;

        output.push_str(&format!("at {}: {}\n", format!("{}:{}:{}", path, start_line + 1, start_col + 1).italic(), self.main_msg));
        output.push_str(&utils::create_snippet(lines, digit_len, start_line, start_col, end_line, end_col, true));

        for (s_msg, s_span) in &self.secondary {
            output.push('\n');

            match s_msg {
                Secondary::Note(m) => {
                    output.push_str(&format!("{}: {m}\n", "note".cyan().bold()));
                    if let Some(s) = s_span {
                        let s_start_line = utils::line_of(s.start, line_starts);
                        let s_start_col = s.start - line_starts[s_start_line];
                        let s_end_line = utils::line_of(s.end, line_starts);
                        let s_end_col = s.end - line_starts[s_end_line];
                        output.push_str(&utils::create_snippet(lines, digit_len, s_start_line, s_start_col, s_end_line, s_end_col, false));
                    }
                },
                Secondary::Help(m) => {
                    output.push_str(&format!("{}: {m}\n", "help".cyan().bold()));
                    if let Some(s) = s_span {
                        let s_start_line = utils::line_of(s.start, line_starts);
                        let s_start_col = s.start - line_starts[s_start_line];
                        let s_end_line = utils::line_of(s.end, line_starts);
                        let s_end_col = s.end - line_starts[s_end_line];
                        output.push_str(&utils::create_snippet(lines, digit_len, s_start_line, s_start_col, s_end_line, s_end_col, false));
                    }
                }
            }
        }
        output.push('\n');

        output
    }
}