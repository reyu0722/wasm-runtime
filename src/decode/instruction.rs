use super::prelude::*;
use crate::core::{BlockType, Expression, Instruction};
use anyhow::{bail, ensure, Context as _, Result};
use std::io::BufRead;

pub trait ReadInstructionExt: BufRead {
    fn read_expr(&mut self) -> Result<Expression> {
        while !self.read_if_equal(0x0b)? {
            self.read_instr().context("failed to read instruction")?;
        }

        Ok(Expression {})
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
            Ok(BlockType::Type(x.try_into()?))
        }
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
            0x0c => Instruction::Br(self.read_u32()?),
            0x0d => Instruction::BrIf(self.read_u32()?),
            0x0e => {
                let vec = read_vec!(self, self.read_u32()?);
                let i = self.read_u32()?;
                Instruction::BrTable(vec, i)
            }
            0x0f => Instruction::Return,
            0x10 => Instruction::Call(self.read_u32()?),
            0x11 => Instruction::CallIndirect {
                ty: self.read_u32()?,
                table: self.read_u32()?,
            },

            // reference instructions
            0xd0 => Instruction::RefNull(self.read_byte()?.try_into()?),
            0xd1 => Instruction::RefIsNull,
            0xd2 => Instruction::RefFunc(self.read_u32()?),

            // parametric instructions
            0x1a => Instruction::Drop,
            0x1b => Instruction::Select,
            0x1c => Instruction::SelectT(read_vec!(self, self.read_value_type()?)),

            // variable instructions
            0x20 | 0x21 | 0x22 | 0x23 | 0x24 => {
                // local.get, local.set, local.tee, global.get, global.set
                self.read_u32()?;
                Instruction::Variable
            }

            // table instructions
            0x25 | 0x26 => {
                // table.get, table.set
                self.read_u32()?;
                Instruction::Table
            }

            // memory instructions
            idx if (0x28..=0x3e).contains(&idx) => {
                self.read_u32()?;
                self.read_u32()?;
                Instruction::Memory
            }
            0x3f | 0x40 => {
                // memory.size, memory.grow
                self.read_u32()?;
                Instruction::Memory
            }

            // numeric instructions
            0x41 => {
                // i32.const
                self.read_signed_leb128(32)?;
                Instruction::Numeric
            }
            0x42 => {
                // i64.const
                self.read_signed_leb128(64)?;
                Instruction::Numeric
            }
            0x43 => {
                // f32.const
                for _ in 0..4 {
                    self.read_byte()?;
                }
                Instruction::Numeric
            }
            0x44 => {
                // f64.const
                for _ in 0..8 {
                    self.read_byte()?;
                }
                Instruction::Numeric
            }
            idx if (0x45..=0xc4).contains(&idx) => Instruction::Numeric,

            0xfc => {
                let kind = self.read_u32()?;
                match kind {
                    // numeric instructions
                    kind if kind <= 0x07 => {}

                    // memory instructions
                    0x08 => {
                        self.read_and_ensure(0x00)
                            .context("invalid memory instruction")?;
                    }
                    0x09 => {
                        self.read_u32()?;
                    }
                    0x10 => {
                        self.read_and_ensure(0x00)
                            .context("invalid memory instruction")?;
                        self.read_and_ensure(0x00)
                            .context("invalid memory instruction")?;
                    }
                    0x11 => {
                        self.read_and_ensure(0x00)
                            .context("invalid memory instruction")?;
                    }

                    // table instructions
                    0x12 | 0x14 => {
                        self.read_u32()?;
                        self.read_u32()?;
                    }
                    0x13 | 0x15 | 0x16 | 0x17 => {
                        self.read_u32()?;
                    }
                    _ => bail!("invalid table instruction: {}", kind),
                }
                Instruction::Memory
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
