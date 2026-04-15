use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueError {
    Overflow,
    InvalidPower,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Value(pub u64);

impl Value {
    pub fn from_int(i: i64) -> Self {
        Self(i as u64)
    }

    pub fn from_float(f: f64) -> Self {
        Self(f.to_bits())
    }

    pub fn from_bool(b: bool) -> Self {
        Self(if b { 1 } else { 0 })
    }

    pub fn unit() -> Self {
        Self(0x0)
    }

    pub fn iadd(&self, other: &Self) -> Result<Self, ValueError> {
        Ok(Self(
            (self.0 as i64)
                .checked_add(other.0 as i64)
                .ok_or(ValueError::Overflow)? as u64,
        ))
    }

    pub fn isub(&self, other: &Self) -> Result<Self, ValueError> {
        Ok(Self(
            (self.0 as i64)
                .checked_sub(other.0 as i64)
                .ok_or(ValueError::Overflow)? as u64,
        ))
    }

    pub fn imul(&self, other: &Self) -> Result<Self, ValueError> {
        Ok(Self(
            (self.0 as i64)
                .checked_mul(other.0 as i64)
                .ok_or(ValueError::Overflow)? as u64,
        ))
    }

    pub fn idiv(&self, other: &Self) -> Result<Self, ValueError> {
        Ok(Self(
            (self.0 as i64)
                .checked_div(other.0 as i64)
                .ok_or(ValueError::Overflow)? as u64,
        ))
    }

    pub fn irem(&self, other: &Self) -> Result<Self, ValueError> {
        Ok(Self(
            (self.0 as i64)
                .checked_rem(other.0 as i64)
                .ok_or(ValueError::Overflow)? as u64,
        ))
    }

    pub fn ipow(&self, other: &Self) -> Result<Self, ValueError> {
        let pow = other.0 as i64;
        if pow < 0 {
            Ok(Self(
                1 / (self.0 as i64)
                    .checked_pow((pow.unsigned_abs() % 64) as u32)
                    .ok_or(ValueError::Overflow)? as u64,
            ))
        } else if pow > 0 {
            Ok(Self(
                (self.0 as i64)
                    .checked_pow((pow as u64 % 64) as u32)
                    .ok_or(ValueError::Overflow)? as u64,
            ))
        } else if self.0 as i64 != 0 {
            Ok(Self::from_int(1))
        } else {
            Err(ValueError::InvalidPower)
        }
    }

    pub fn fadd(&self, other: &Self) -> Result<Self, ValueError> {
        Ok(Self(
            (f64::from_bits(self.0) + f64::from_bits(other.0)).to_bits(),
        ))
    }

    pub fn fsub(&self, other: &Self) -> Result<Self, ValueError> {
        Ok(Self(
            (f64::from_bits(self.0) - f64::from_bits(other.0)).to_bits(),
        ))
    }

    pub fn fmul(&self, other: &Self) -> Result<Self, ValueError> {
        Ok(Self(
            (f64::from_bits(self.0) * f64::from_bits(other.0)).to_bits(),
        ))
    }

    pub fn fdiv(&self, other: &Self) -> Result<Self, ValueError> {
        Ok(Self(
            (f64::from_bits(self.0) / f64::from_bits(other.0)).to_bits(),
        ))
    }

    pub fn frem(&self, other: &Self) -> Result<Self, ValueError> {
        Ok(Self(
            (f64::from_bits(self.0) % f64::from_bits(other.0)).to_bits(),
        ))
    }

    pub fn fpow(&self, other: &Self) -> Result<Self, ValueError> {
        Ok(Self(
            (f64::from_bits(self.0).powf(f64::from_bits(other.0))).to_bits(),
        ))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstantId(pub usize);

impl fmt::Debug for ConstantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "${}", self.0)
    }
}
