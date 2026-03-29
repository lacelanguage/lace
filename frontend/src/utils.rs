use tinycolor::Colorize;

pub fn line_starts(src: &str) -> Vec<usize> {
    let mut l = vec![0];

    for (pos, ch) in src.char_indices() {
        if ch == '\n' {
            l.push(pos + 1);
        }
    }

    l
}

pub fn line_of(pos: usize, line_starts: &[usize]) -> usize {
    match line_starts.binary_search(&pos) {
        Ok(line) => line,
        Err(line) => line - 1,
    }
}

pub fn create_snippet(lines: &[&str], digit_len: usize, start_line: usize, start_col: usize, end_line: usize, end_col: usize, main_or_sec: bool) -> String {
    let mut output = String::new();

    for (idx, l) in (start_line..=end_line).enumerate() {
        if idx > 0 {
            output.push('\n');
        }

        let line_text = lines[l];
        if l == start_line && l == end_line {
            let prev = &line_text[..start_col];
            let main = &line_text[start_col..end_col];
            let after = &line_text[end_col..];
            output.push_str(&format!(" {:>digit_len$} | {prev}{}{after}\n", l + 1, if main_or_sec { main.red() } else { main.cyan() }));
            output.push_str(&format!(" {:>digit_len$} | ", ""));
            output.push_str(&" ".repeat(start_col));
            if main_or_sec {
                output.push_str(&"^".repeat(end_col - start_col).red().to_string());
            } else {
                output.push_str(&"¯".repeat(end_col - start_col).cyan().to_string());
            }
        } else if l == start_line {
            let prev = &line_text[..start_col];
            let main = &line_text[start_col..];
            output.push_str(&format!(" {:>digit_len$} | {prev}{}\n", l + 1, if main_or_sec { main.red() } else { main.cyan() }));
            output.push_str(&format!(" {:>digit_len$} | ", ""));
            output.push_str(&" ".repeat(start_col));
            if main_or_sec {
                output.push_str(&"^".repeat(line_text.len() - start_col).red().to_string());
            } else {
                output.push_str(&"¯".repeat(line_text.len() - start_col).cyan().to_string());
            }
        } else if l == end_line {
            let main = &line_text[..end_col];
            let after = &line_text[end_col..];
            output.push_str(&format!(" {:>digit_len$} | {}{after}\n", l + 1, if main_or_sec { main.red() } else { main.cyan() }));
            output.push_str(&format!(" {:>digit_len$} | ", ""));
            if main_or_sec {
                output.push_str(&"^".repeat(end_col).red().to_string());
            } else {
                output.push_str(&"¯".repeat(end_col).cyan().to_string());
            }
        } else {
            output.push_str(&format!(" {:>digit_len$} | {}\n", l + 1, if main_or_sec { line_text.red() } else { line_text.cyan() }));
            output.push_str(&format!(" {:>digit_len$} | ", ""));
            if main_or_sec {
                output.push_str(&"^".repeat(line_text.len()).red().to_string());
            } else {
                output.push_str(&"¯".repeat(line_text.len()).cyan().to_string());
            }
        }
    }

    output
}