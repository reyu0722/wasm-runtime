use super::prelude::*;
use crate::core::{BlockType, Expression, IBinOp, IRelOp, IUnOp, Instruction, MemArg};
use anyhow::{bail, ensure, Context as _, Result};
use std::io::BufRead;

pub trait ReadInstructionExt: BufRead {
    fn read_expr(&mut self) -> Result<Expression> {
        let mut vec = Vec::new();
        while !self.read_if_equal(0x0b)? {
            vec.push(self.read_instr().context("failed to read instruction")?);
        }

        Ok(Expression { instructions: vec })
    }

    fn read_block_type(&mut self) -> Result<BlockType> {
        // TODO: fix
        let mut slice = &self.fill_buf()?[0..1];
        if let Ok(value) = slice.read_value_type() {
            self.consume(1);
            Ok(BlockType::ValType(Some(value)))
        } else if self.read_if_equal(0x40)? {
            Ok(BlockType::ValType(None))
        } else {
            let x = self.read_signed_leb128(33)?;
            ensure!(x >= 0, "invalid block type");
            Ok(BlockType::Type(u32::try_from(x)?.into()))
        }
    }

    fn read_mem_arg(&mut self) -> Result<MemArg> {
        let align = self.read_u32().context("failed to read align")?;
        let offset = self.read_u32().context("failed to read offset")?;
        Ok(MemArg { align, offset })
    }

    fn read_instr(&mut self) -> Result<Instruction> {
        let opcode = self.read_byte().context("failed to read opcode")?;
        let instr = match opcode {
            // control instructions
            0x00 => Instruction::Unreachable,
            0x01 => Instruction::Nop,
            0x02 => {
                let block_type = self.read_block_type()?;

                let mut instructions = Vec::new();
                while !self.read_if_equal(0x0b)? {
                    instructions.push(self.read_instr()?);
                }
                Instruction::Block {
                    block_type,
                    instructions,
                }
            }
            0x03 => {
                let block_type = self.read_block_type()?;

                let mut instructions = Vec::new();
                while !self.read_if_equal(0x0b)? {
                    instructions.push(self.read_instr()?);
                }
                Instruction::Loop {
                    block_type,
                    instructions,
                }
            }
            0x04 => {
                let block_type = self.read_block_type()?;

                let mut instructions = Vec::new();
                let mut else_instructions = Vec::new();

                while !self.read_if_equal(0x0b)? {
                    if self.read_if_equal(0x05)? {
                        while !self.read_if_equal(0x0b)? {
                            else_instructions.push(self.read_instr()?);
                        }
                        break;
                    }
                    instructions.push(self.read_instr()?);
                }
                Instruction::If {
                    block_type,
                    instructions,
                    else_instructions,
                }
            }
            0x0c => Instruction::Br(self.read_u32()?.into()),
            0x0d => Instruction::BrIf(self.read_u32()?.into()),
            0x0e => {
                let vec = read_vec!(self, self.read_u32()?.into());
                let i = self.read_u32()?.into();
                Instruction::BrTable(vec, i)
            }
            0x0f => Instruction::Return,
            0x10 => Instruction::Call(self.read_u32()?.into()),
            0x11 => Instruction::CallIndirect {
                ty: self.read_u32()?.into(),
                table: self.read_u32()?.into(),
            },

            // reference instructions
            0xd0 => Instruction::RefNull(self.read_byte()?.try_into()?),
            0xd1 => Instruction::RefIsNull,
            0xd2 => Instruction::RefFunc(self.read_u32()?.into()),

            // parametric instructions
            0x1a => Instruction::Drop,
            0x1b => Instruction::Select(Vec::new()),
            0x1c => Instruction::Select(read_vec!(self, self.read_value_type()?)),

            // variable instructions
            0x20 => Instruction::LocalGet(self.read_u32()?.into()),
            0x21 => Instruction::LocalSet(self.read_u32()?.into()),
            0x22 => Instruction::LocalTee(self.read_u32()?.into()),
            0x23 => Instruction::GlobalGet(self.read_u32()?.into()),
            0x24 => Instruction::GlobalSet(self.read_u32()?.into()),

            // table instructions

            // memory instructions
            0x28 => Instruction::I32Load(self.read_mem_arg()?),
            0x29 => Instruction::I64Load(self.read_mem_arg()?),
            0x2a => Instruction::F32Load(self.read_mem_arg()?),
            0x2b => Instruction::F64Load(self.read_mem_arg()?),
            0x2c => Instruction::I32Load8S(self.read_mem_arg()?),
            0x2d => Instruction::I32Load8U(self.read_mem_arg()?),
            0x2e => Instruction::I32Load16S(self.read_mem_arg()?),
            0x2f => Instruction::I32Load16U(self.read_mem_arg()?),
            0x30 => Instruction::I64Load8S(self.read_mem_arg()?),
            0x31 => Instruction::I64Load8U(self.read_mem_arg()?),
            0x32 => Instruction::I64Load16S(self.read_mem_arg()?),
            0x33 => Instruction::I64Load16U(self.read_mem_arg()?),
            0x34 => Instruction::I64Load32S(self.read_mem_arg()?),
            0x35 => Instruction::I64Load32U(self.read_mem_arg()?),
            0x36 => Instruction::I32Store(self.read_mem_arg()?),
            0x37 => Instruction::I64Store(self.read_mem_arg()?),
            0x38 => Instruction::F32Store(self.read_mem_arg()?),
            0x39 => Instruction::F64Store(self.read_mem_arg()?),
            0x3a => Instruction::I32Store8(self.read_mem_arg()?),
            0x3b => Instruction::I32Store16(self.read_mem_arg()?),
            0x3c => Instruction::I64Store8(self.read_mem_arg()?),
            0x3d => Instruction::I64Store16(self.read_mem_arg()?),
            0x3e => Instruction::I64Store32(self.read_mem_arg()?),
            0x3f => {
                self.read_and_ensure(0x00)?;
                Instruction::MemorySize
            }
            0x40 => {
                self.read_and_ensure(0x00)?;
                Instruction::MemoryGrow
            }

            // numeric instructions
            0x41 => {
                let v = self.read_signed_leb128(32)?.try_into()?;
                Instruction::I32Const(v)
            }
            0x42 => {
                let v = self.read_signed_leb128(64)?;
                Instruction::I64Const(v)
            }
            0x43 => {
                // f32.const
                for _ in 0..4 {
                    self.read_byte()?;
                }
                Instruction::Numeric(opcode.into())
            }
            0x44 => {
                // f64.const
                for _ in 0..8 {
                    self.read_byte()?;
                }
                Instruction::Numeric(opcode.into())
            }

            0xc0 => Instruction::I32Extend8S,
            0xc1 => Instruction::I32Extend16S,
            0x45 => Instruction::I32Eqz,
            idx if (0x46..=0x4f).contains(&idx) => {
                let op = match idx {
                    0x46 => IRelOp::Eq,
                    0x47 => IRelOp::Ne,
                    0x48 => IRelOp::LtS,
                    0x49 => IRelOp::LtU,
                    0x4a => IRelOp::GtS,
                    0x4b => IRelOp::GtU,
                    0x4c => IRelOp::LeS,
                    0x4d => IRelOp::LeU,
                    0x4e => IRelOp::GeS,
                    0x4f => IRelOp::GeU,
                    _ => unreachable!("checked above"),
                };

                Instruction::I32RelOp(op)
            }
            0x50 => Instruction::I64Eqz,
            idx if (0x51..=0x5a).contains(&idx) => {
                let op = match idx {
                    0x51 => IRelOp::Eq,
                    0x52 => IRelOp::Ne,
                    0x53 => IRelOp::LtS,
                    0x54 => IRelOp::LtU,
                    0x55 => IRelOp::GtS,
                    0x56 => IRelOp::GtU,
                    0x57 => IRelOp::LeS,
                    0x58 => IRelOp::LeU,
                    0x59 => IRelOp::GeS,
                    0x5a => IRelOp::GeU,
                    _ => unreachable!("checked above"),
                };

                Instruction::I64RelOp(op)
            }
            idx if (0x67..=0x69).contains(&idx) => {
                let op = match idx {
                    0x67 => IUnOp::Clz,
                    0x68 => IUnOp::Ctz,
                    0x69 => IUnOp::Popcnt,
                    _ => unreachable!("checked above"),
                };

                Instruction::I32UnOp(op)
            }
            idx if (0x6a..=0x78).contains(&idx) => {
                let op = match idx {
                    0x6a => IBinOp::Add,
                    0x6b => IBinOp::Sub,
                    0x6c => IBinOp::Mul,
                    0x6d => IBinOp::DivS,
                    0x6e => IBinOp::DivU,
                    0x6f => IBinOp::RemS,
                    0x70 => IBinOp::RemU,
                    0x71 => IBinOp::And,
                    0x72 => IBinOp::Or,
                    0x73 => IBinOp::Xor,
                    0x74 => IBinOp::Shl,
                    0x75 => IBinOp::ShrS,
                    0x76 => IBinOp::ShrU,
                    0x77 => IBinOp::Rotl,
                    0x78 => IBinOp::Rotr,
                    _ => unreachable!("checked above"),
                };

                Instruction::I32BinOp(op)
            }

            idx if (0x79..=0x7b).contains(&idx) => {
                let op = match idx {
                    0x79 => IUnOp::Clz,
                    0x7a => IUnOp::Ctz,
                    0x7b => IUnOp::Popcnt,
                    _ => unreachable!("checked above"),
                };

                Instruction::I64UnOp(op)
            }
            idx if (0x7c..=0x8a).contains(&idx) => {
                let op = match idx {
                    0x7c => IBinOp::Add,
                    0x7d => IBinOp::Sub,
                    0x7e => IBinOp::Mul,
                    0x7f => IBinOp::DivS,
                    0x80 => IBinOp::DivU,
                    0x81 => IBinOp::RemS,
                    0x82 => IBinOp::RemU,
                    0x83 => IBinOp::And,
                    0x84 => IBinOp::Or,
                    0x85 => IBinOp::Xor,
                    0x86 => IBinOp::Shl,
                    0x87 => IBinOp::ShrS,
                    0x88 => IBinOp::ShrU,
                    0x89 => IBinOp::Rotl,
                    0x8a => IBinOp::Rotr,
                    _ => unreachable!("checked above"),
                };

                Instruction::I64BinOp(op)
            }
            0xc2 => Instruction::I64Extend8S,
            0xc3 => Instruction::I64Extend16S,
            0xc4 => Instruction::I64Extend32S,

            idx if (0x45..=0xc4).contains(&idx) => Instruction::Numeric(opcode.into()),

            0xfc => {
                let kind = self.read_u32()?;
                match kind {
                    // numeric instructions
                    kind if kind <= 0x07 => Instruction::Numeric(opcode.into()),

                    // memory instructions
                    0x08 => {
                        let idx = self.read_u32()?.into();
                        self.read_and_ensure(0x00)?;
                        Instruction::MemoryInit(idx)
                    }
                    0x09 => {
                        let idx = self.read_u32()?.into();
                        Instruction::DataDrop(idx)
                    }
                    0x10 => {
                        self.read_and_ensure(0x00)
                            .context("invalid memory instruction")?;
                        self.read_and_ensure(0x00)
                            .context("invalid memory instruction")?;
                        Instruction::MemoryCopy
                    }
                    0x11 => {
                        self.read_and_ensure(0x00)
                            .context("invalid memory instruction")?;
                        Instruction::MemoryFill
                    }

                    // table instructions
                    _ => bail!("invalid table instruction: {}", kind),
                }
            }

            // vector instructions
            0xfd => {
                let kind = self.read_u32()?;
                match kind {
                    kind if kind <= 11 || kind == 92 || kind == 93 => {
                        self.read_u32()?;
                        self.read_u32()?;
                    }
                    kind if 84 <= kind || kind <= 91 => {
                        self.read_u32()?;
                        self.read_u32()?;
                        self.read_u32()?;
                    }
                    12 => {
                        for _ in 0..16 {
                            self.read_byte()?;
                        }
                    }
                    13 => {
                        for _ in 0..16 {
                            self.read_u32()?;
                        }
                    }
                    kind if (21..=34).contains(&kind) => {
                        self.read_u32()?;
                    }
                    _ => {}
                }
                Instruction::Vector
            }

            _ => bail!("invalid opcode: {}", opcode),
        };

        Ok(instr)
    }
}

impl<R: BufRead + ?Sized> ReadInstructionExt for R {}
