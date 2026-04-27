pub mod value;
pub mod inst;
pub mod function;

use std::sync::Arc;
use value::{Value, ValueError};
use inst::Inst;

pub struct Program {
    pub code: Vec<Inst>,
    pub consts: Vec<u8>,
    pub statics: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VMState {
    Running(usize),
    Sleeping {
        continue_at: usize,
        remaining: f32,
    },
    Halted,
}

pub enum VMError {
    ValueError(ValueError)
}

pub struct LaceVM {
    code: Arc<[Inst]>,
    stack: Vec<u8>,
    mem: Vec<u8>,
    registers: [Value; 16],

    state: VMState
}

impl LaceVM {
    pub const ZR: u8 = 0x0;
    pub const R0: u8 = 0x1;
    pub const R1: u8 = 0x2;
    pub const R2: u8 = 0x3;
    pub const R3: u8 = 0x4;
    pub const R4: u8 = 0x5;
    pub const R5: u8 = 0x6;
    pub const R6: u8 = 0x7;
    pub const R7: u8 = 0x8;
    pub const R8: u8 = 0x9;
    pub const R9: u8 = 0xA;
    pub const R10: u8 = 0xB;
    pub const R11: u8 = 0xC;
    pub const RA: u8 = 0xD;
    pub const SP: u8 = 0xE;
    pub const FP: u8 = 0xF;

    fn write_reg(&mut self, dest: u8, value: Value) {
        if dest != 0 {
            self.registers[dest as usize] = value;
        }
    }

    fn read_reg(&mut self, src: u8) -> Value {
        self.registers.get(src as usize).copied().unwrap_or(Value::unit())
    }

    pub fn new(program: Program) -> Self {
        Self {
            code: program.code.into(),
            stack: vec![0; 1_056_784],
            mem: [program.consts, program.statics].concat(),
            registers: [Value::unit(); 16],
            state: VMState::Halted
        }
    }

    pub fn run(&mut self) -> Result<(), VMError> {
        use std::time::Instant;

        self.state = VMState::Running(0);
        let mut prev = Instant::now();
        while self.state != VMState::Halted {
            self.step(prev.elapsed().as_secs_f32())?;
            prev = Instant::now();
        }

        Ok(())
    }

    fn step(&mut self, dt: f32) -> Result<(), VMError> {
        match &mut self.state {
            VMState::Running(pc) => {
                if *pc >= self.code.len() {
                    self.state = VMState::Halted;
                    return Ok(());
                }
                let inst = self.code[*pc];
                *pc += 1;

                match inst {
                    Inst::Mov(dest, src) => {
                        let value = self.read_reg(src);
                        self.write_reg(dest, value);
                    },
                    Inst::IAdd(dest, src1, src2) => {
                        let value = self.read_reg(src1).iadd(&self.read_reg(src2)).map_err(VMError::ValueError)?;
                        self.write_reg(
                            dest,
                            value
                        )
                    },
                    Inst::ISub(dest, src1, src2) => {
                        let value = self.read_reg(src1).isub(&self.read_reg(src2)).map_err(VMError::ValueError)?;
                        self.write_reg(
                            dest,
                            value
                        )
                    },
                    Inst::IMul(dest, src1, src2) => {
                        let value = self.read_reg(src1).imul(&self.read_reg(src2)).map_err(VMError::ValueError)?;
                        self.write_reg(
                            dest,
                            value
                        )
                    },
                    Inst::IDiv(dest, src1, src2) => {
                        let value = self.read_reg(src1).idiv(&self.read_reg(src2)).map_err(VMError::ValueError)?;
                        self.write_reg(
                            dest,
                            value
                        )
                    },
                    Inst::IRem(dest, src1, src2) => {
                        let value = self.read_reg(src1).irem(&self.read_reg(src2)).map_err(VMError::ValueError)?;
                        self.write_reg(
                            dest,
                            value
                        )
                    },
                    Inst::IPow(dest, src1, src2) => {
                        let value = self.read_reg(src1).ipow(&self.read_reg(src2)).map_err(VMError::ValueError)?;
                        self.write_reg(
                            dest,
                            value
                        )
                    },
                    Inst::FAdd(dest, src1, src2) => {
                        let value = self.read_reg(src1).fadd(&self.read_reg(src2)).map_err(VMError::ValueError)?;
                        self.write_reg(
                            dest,
                            value
                        )
                    },
                    Inst::FSub(dest, src1, src2) => {
                        let value = self.read_reg(src1).fsub(&self.read_reg(src2)).map_err(VMError::ValueError)?;
                        self.write_reg(
                            dest,
                            value
                        )
                    },
                    Inst::FMul(dest, src1, src2) => {
                        let value = self.read_reg(src1).fmul(&self.read_reg(src2)).map_err(VMError::ValueError)?;
                        self.write_reg(
                            dest,
                            value
                        )
                    },
                    Inst::FDiv(dest, src1, src2) => {
                        let value = self.read_reg(src1).fdiv(&self.read_reg(src2)).map_err(VMError::ValueError)?;
                        self.write_reg(
                            dest,
                            value
                        )
                    },
                    Inst::FRem(dest, src1, src2) => {
                        let value = self.read_reg(src1).frem(&self.read_reg(src2)).map_err(VMError::ValueError)?;
                        self.write_reg(
                            dest,
                            value
                        )
                    },
                    Inst::FPow(dest, src1, src2) => {
                        let value = self.read_reg(src1).fpow(&self.read_reg(src2)).map_err(VMError::ValueError)?;
                        self.write_reg(
                            dest,
                            value
                        )
                    },
                    Inst::SbS(dest, src) => {
                        let addr = self.read_reg(dest).0 as usize;
                        if addr >= self.stack.len() {
                            self.stack.resize(addr+1, 0);
                        }
                        let val = self.read_reg(src).0;
                        self.stack[addr] = (val & 0xFF) as u8;
                    },
                    Inst::ShS(dest, src) => {
                        let addr = self.read_reg(dest).0 as usize;
                        if addr >= self.stack.len() {
                            self.stack.resize(addr+2, 0);
                        }
                        let val = self.read_reg(src).0;
                        self.stack[addr] = (val & 0xFF) as u8;
                        self.stack[addr+1] = (val >> 8) as u8;
                    },
                    Inst::SwS(dest, src) => {
                        let addr = self.read_reg(dest).0 as usize;
                        if addr >= self.stack.len() {
                            self.stack.resize(addr+4, 0);
                        }
                        let val = self.read_reg(src).0;
                        self.stack[addr] = (val & 0xFF) as u8;
                        self.stack[addr+1] = ((val >> 8) & 0xFF) as u8;
                        self.stack[addr+2] = ((val >> 16) & 0xFF) as u8;
                        self.stack[addr+3] = (val >> 24) as u8;
                    },
                    Inst::SdS(dest, src) => {
                        let addr = self.read_reg(dest).0 as usize;
                        if addr >= self.stack.len() {
                            self.stack.resize(addr+8, 0);
                        }
                        let val = self.read_reg(src).0;
                        self.stack[addr] = (val & 0xFF) as u8;
                        self.stack[addr+1] = ((val >> 8) & 0xFF) as u8;
                        self.stack[addr+2] = ((val >> 16) & 0xFF) as u8;
                        self.stack[addr+3] = ((val >> 24) & 0xFF) as u8;
                        self.stack[addr+4] = ((val >> 32) & 0xFF) as u8;
                        self.stack[addr+5] = ((val >> 40) & 0xFF) as u8;
                        self.stack[addr+6] = ((val >> 48) & 0xFF) as u8;
                        self.stack[addr+7] = (val >> 56) as u8;
                    },
                    Inst::LbS(dest, src) => {
                        let addr = self.read_reg(src).0 as usize;
                        if addr >= self.stack.len() {
                            self.stack.resize(addr+1, 0);
                        }
                        let val = Value::from_int(self.stack[addr] as i64);
                        self.write_reg(dest, val);
                    },
                    Inst::LhS(dest, src) => {
                        let addr = self.read_reg(src).0 as usize;
                        if addr >= self.stack.len() {
                            self.stack.resize(addr+2, 0);
                        }
                        let val = Value::from_int(u16::from_le_bytes(self.stack[addr..addr+1].try_into().unwrap()) as i64);
                        self.write_reg(dest, val);
                    },
                    Inst::LwS(dest, src) => {
                        let addr = self.read_reg(src).0 as usize;
                        if addr >= self.stack.len() {
                            self.stack.resize(addr+4, 0);
                        }
                        let val = Value::from_int(u32::from_le_bytes(self.stack[addr..addr+3].try_into().unwrap()) as i64);
                        self.write_reg(dest, val);
                    },
                    Inst::LdS(dest, src) => {
                        let addr = self.read_reg(src).0 as usize;
                        if addr >= self.stack.len() {
                            self.stack.resize(addr+8, 0);
                        }
                        let val = Value::from_int(u64::from_le_bytes(self.stack[addr..addr+7].try_into().unwrap()) as i64);
                        self.write_reg(dest, val);
                    },
                    Inst::LbuS(dest, src) => {
                        let addr = self.read_reg(src).0 as usize;
                        if addr >= self.stack.len() {
                            self.stack.resize(addr+1, 0);
                        }
                        let val = Value::from_int(self.stack[addr] as u64 as i64);
                        self.write_reg(dest, val);
                    },
                    Inst::LhuS(dest, src) => {
                        let addr = self.read_reg(src).0 as usize;
                        if addr >= self.stack.len() {
                            self.stack.resize(addr+2, 0);
                        }
                        let val = Value::from_int(u16::from_le_bytes(self.stack[addr..addr+1].try_into().unwrap()) as u64 as i64);
                        self.write_reg(dest, val);
                    },
                    Inst::LwuS(dest, src) => {
                        let addr = self.read_reg(src).0 as usize;
                        if addr >= self.stack.len() {
                            self.stack.resize(addr+4, 0);
                        }
                        let val = Value::from_int(u32::from_le_bytes(self.stack[addr..addr+3].try_into().unwrap()) as u64 as i64);
                        self.write_reg(dest, val);
                    },
                    Inst::SbM(dest, src) => {
                        let addr = self.read_reg(dest).0 as usize;
                        if addr >= self.mem.len() {
                            self.mem.resize(addr+1, 0);
                        }
                        let val = self.read_reg(src).0;
                        self.mem[addr] = (val & 0xFF) as u8;
                    },
                    Inst::ShM(dest, src) => {
                        let addr = self.read_reg(dest).0 as usize;
                        if addr >= self.mem.len() {
                            self.mem.resize(addr+2, 0);
                        }
                        let val = self.read_reg(src).0;
                        self.mem[addr] = (val & 0xFF) as u8;
                        self.mem[addr+1] = (val >> 8) as u8;
                    },
                    Inst::SwM(dest, src) => {
                        let addr = self.read_reg(dest).0 as usize;
                        if addr >= self.mem.len() {
                            self.mem.resize(addr+4, 0);
                        }
                        let val = self.read_reg(src).0;
                        self.mem[addr] = (val & 0xFF) as u8;
                        self.mem[addr+1] = ((val >> 8) & 0xFF) as u8;
                        self.mem[addr+2] = ((val >> 16) & 0xFF) as u8;
                        self.mem[addr+3] = (val >> 24) as u8;
                    },
                    Inst::SdM(dest, src) => {
                        let addr = self.read_reg(dest).0 as usize;
                        if addr >= self.mem.len() {
                            self.mem.resize(addr+8, 0);
                        }
                        let val = self.read_reg(src).0;
                        self.mem[addr] = (val & 0xFF) as u8;
                        self.mem[addr+1] = ((val >> 8) & 0xFF) as u8;
                        self.mem[addr+2] = ((val >> 16) & 0xFF) as u8;
                        self.mem[addr+3] = ((val >> 24) & 0xFF) as u8;
                        self.mem[addr+4] = ((val >> 32) & 0xFF) as u8;
                        self.mem[addr+5] = ((val >> 40) & 0xFF) as u8;
                        self.mem[addr+6] = ((val >> 48) & 0xFF) as u8;
                        self.mem[addr+7] = (val >> 56) as u8;
                    },
                    Inst::LbM(dest, src) => {
                        let addr = self.read_reg(src).0 as usize;
                        if addr >= self.mem.len() {
                            self.mem.resize(addr+1, 0);
                        }
                        let val = Value::from_int(self.mem[addr] as i64);
                        self.write_reg(dest, val);
                    },
                    Inst::LhM(dest, src) => {
                        let addr = self.read_reg(src).0 as usize;
                        if addr >= self.mem.len() {
                            self.mem.resize(addr+2, 0);
                        }
                        let val = Value::from_int(u16::from_le_bytes(self.mem[addr..addr+1].try_into().unwrap()) as i64);
                        self.write_reg(dest, val);
                    },
                    Inst::LwM(dest, src) => {
                        let addr = self.read_reg(src).0 as usize;
                        if addr >= self.mem.len() {
                            self.mem.resize(addr+4, 0);
                        }
                        let val = Value::from_int(u32::from_le_bytes(self.mem[addr..addr+3].try_into().unwrap()) as i64);
                        self.write_reg(dest, val);
                    },
                    Inst::LdM(dest, src) => {
                        let addr = self.read_reg(src).0 as usize;
                        if addr >= self.mem.len() {
                            self.mem.resize(addr+8, 0);
                        }
                        let val = Value::from_int(u64::from_le_bytes(self.mem[addr..addr+7].try_into().unwrap()) as i64);
                        self.write_reg(dest, val);
                    },
                    Inst::LbuM(dest, src) => {
                        let addr = self.read_reg(src).0 as usize;
                        if addr >= self.mem.len() {
                            self.mem.resize(addr+1, 0);
                        }
                        let val = Value::from_int(self.mem[addr] as u64 as i64);
                        self.write_reg(dest, val);
                    },
                    Inst::LhuM(dest, src) => {
                        let addr = self.read_reg(src).0 as usize;
                        if addr >= self.mem.len() {
                            self.mem.resize(addr+2, 0);
                        }
                        let val = Value::from_int(u16::from_le_bytes(self.mem[addr..addr+1].try_into().unwrap()) as u64 as i64);
                        self.write_reg(dest, val);
                    },
                    Inst::LwuM(dest, src) => {
                        let addr = self.read_reg(src).0 as usize;
                        if addr >= self.mem.len() {
                            self.mem.resize(addr+4, 0);
                        }
                        let val = Value::from_int(u32::from_le_bytes(self.mem[addr..addr+3].try_into().unwrap()) as u64 as i64);
                        self.write_reg(dest, val);
                    },
                }
            },
            VMState::Sleeping {
                continue_at,
                remaining
            } => {
                if *remaining <= 0.0 {
                    self.state = VMState::Running(*continue_at);
                } else {
                    *remaining -= dt;
                }
            },
            VMState::Halted => return Ok(()),
        }

        Ok(())
    }
}