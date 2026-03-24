#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Op {
    Plus, Minus, Star, Slash, Modulo, Power
}

impl Op {
    pub fn binding_power(&self) -> (usize, usize) {
        match self {
            Self::Plus | Self::Minus => (10, 11),
            Self::Star | Self::Slash | Self::Modulo => (20, 21),
            Self::Power => (31, 30),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Star => "*",
            Self::Slash => "/",
            Self::Modulo => "%",
            Self::Power => "**",
        }
    }
}