use super::prelude::*;
use crate::core::{BlockType, Expression};
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

    fn read_instr(&mut self) -> Result<()> {
        let opcode = self.read_byte().context("failed to read opcode")?;
        match opcode {
            // control instructions
            0x00 => (), // unreachable
            0x01 => (), // nop
            0x02 | 0x03 => {
                // block, loop
                self.read_block_type()?;
                while !self.read_if_equal(0x0b)? {
                    self.read_instr()?;
                }
            }
            0x04 => {
                // if
                self.read_block_type()?;
                while !self.read_if_equal(0x0b)? {
                    if self.read_if_equal(0x05)? {
                        while !self.read_if_equal(0x0b)? {
                            self.read_instr()?;
                        }
                        break;
                    }
                    self.read_instr()?;
                }
            }
            0x0c | 0x0d => {
                // br, br_if
                self.read_u32()?;
            }
            0x0e => {
                // br_table
                read_vec!(self, self.read_u32()?);
                self.read_u32()?;
            }
            0x0f => {
                // return
            }
            0x10 => {
                // call
                self.read_u32()?;
            }
            0x11 => {
                // call_indirect
                self.read_u32()?;
                self.read_u32()?;
            }

            // reference instructions
            0xd0 => {
                // ref.null
                let ref_type = self.read_byte()?;
                ensure!(
                    ref_type == 0x70 || ref_type == 0x6f,
                    "invalid ref.null type"
                );
            }
            0xd1 => {
                // ref.is_null
            }
            0xd2 => {
                // ref.func
                self.read_u32()?;
            }

            // parametric instructions
            0x1a | 0x1b => {
                // drop, select
            }
            0x1c => {
                // select t*
                read_vec!(self, self.read_value_type()?);
            }

            // variable instructions
            0x20 | 0x21 | 0x22 | 0x23 | 0x24 => {
                // local.get, local.set, local.tee, global.get, global.set
                self.read_u32()?;
            }

            // table instructions
            0x25 | 0x26 => {
                // table.get, table.set
                self.read_u32()?;
            }

            // memory instructions
            idx if (0x28..=0x3e).contains(&idx) => {
                self.read_u32()?;
                self.read_u32()?;
            }
            0x3f | 0x40 => {
                // memory.size, memory.grow
                self.read_u32()?;
            }

            // numeric instructions
            0x41 => {
                // i32.const
                self.read_signed_leb128(32)?;
            }
            0x42 => {
                // i64.const
                self.read_signed_leb128(64)?;
            }
            0x43 => {
                // f32.const
                for _ in 0..4 {
                    self.read_byte()?;
                }
            }
            0x44 => {
                // f64.const
                for _ in 0..8 {
                    self.read_byte()?;
                }
            }
            idx if (0x45..=0xc4).contains(&idx) => {}

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
            }

            _ => bail!("invalid opcode: {}", opcode),
        }

        Ok(())
    }
}

impl<R: BufRead + ?Sized> ReadInstructionExt for R {}
