#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn extend(&mut self, other: Span) {
        self.end = other.end;
    }

    pub fn splat_to_end(&mut self) {
        self.start = self.end;
        self.end += 1;
    }

    pub fn empty() -> Self {
        Self {
            start: 0usize,
            end: 1usize
        }
    }
}