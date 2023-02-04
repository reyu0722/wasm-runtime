use super::{types::ReadTypeExt, value::ReadValueExt};
use anyhow::{bail, ensure, Result};
use std::io::BufRead;

pub trait ReadInstructionExt: BufRead {
    fn read_expr(&mut self) -> Result<()> {
        Ok(())
    }

    fn read_block_type(&mut self) -> Result<()> {
        let mut block_type = &self.fill_buf()?[0..1];
        if let Ok(()) = block_type.read_value_type() {
            self.consume(1);
            Ok(())
        } else if block_type[0] == 0x40 {
            self.consume(1);
            Ok(()) // empty
        } else {
            self.read_signed_leb128(33)?;
            Ok(())
        }
    }

    fn read_instr(&mut self) -> Result<()> {
        let opcode = self.read_u8()?;
        match opcode {
            // control instructions
            0x00 => (), // unreachable
            0x01 => (), // nop
            0x02 | 0x03 => {
                // block, loop
                self.read_block_type()?;
                loop {
                    if self.fill_buf()?[0] == 0x0b {
                        self.consume(1);
                        break;
                    }
                    self.read_instr()?;
                }
            }
            0x04 => {
                // if
                self.read_block_type()?;
                loop {
                    if self.fill_buf()?[0] == 0x0b {
                        self.consume(1);
                        break;
                    } else if self.fill_buf()?[0] == 0x05 {
                        self.consume(1);
                        loop {
                            if self.fill_buf()?[0] == 0x0b {
                                self.consume(1);
                                break;
                            }
                            self.read_instr()?;
                        }
                        break;
                    }
                    self.read_instr()?;
                }
            }
            0x0c | 0x0d => {
                // br, br_if
                self.read_unsigned_leb128(32)?;
            }
            0x0e => {
                // br_table
                let size = self.read_unsigned_leb128(32)?;
                for _ in 0..size {
                    self.read_unsigned_leb128(32)?;
                }
                self.read_unsigned_leb128(32)?;
            }
            0x0f => {
                // return
            }
            0x10 => {
                // call
                self.read_unsigned_leb128(32)?;
            }
            0x11 => {
                // call_indirect
                self.read_unsigned_leb128(32)?;
                self.read_unsigned_leb128(32)?;
            }

            // reference instructions
            0xd0 => {
                // ref.null
                let ref_type = self.read_u8()?;
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
                self.read_unsigned_leb128(32)?;
            }

            // parametric instructions
            0x1a | 0x1b => {
                // drop, select
            }
            0x1c => {
                // select t*
                let size = self.read_unsigned_leb128(32)?;
                for _ in 0..size {
                    self.read_value_type()?;
                }
            }

            // variable instructions
            0x20 | 0x21 | 0x22 | 0x23 | 0x24 => {
                // local.get, local.set, local.tee, global.get, global.set
                self.read_unsigned_leb128(32)?;
            }

            // table instructions
            0x25 | 0x26 => {
                // table.get, table.set
                self.read_unsigned_leb128(32)?;
            }

            // memory instructions
            idx if 0x28 <= idx && idx <= 0x40 => {
                self.read_unsigned_leb128(32)?;
                self.read_unsigned_leb128(32)?;
            }

            0xfc => {
                let kind = self.read_unsigned_leb128(32)?;
                match kind {
                    // memory instructions
                    0x08 => {
                        self.read_unsigned_leb128(32)?;
                        ensure!(
                            self.read_unsigned_leb128(32)? == 0,
                            "invalid memory instruction"
                        )
                    }
                    0x09 => {
                        self.read_unsigned_leb128(32)?;
                    }
                    0x10 => {
                        ensure!(
                            self.read_unsigned_leb128(32)? == 0,
                            "invalid memory instruction"
                        );
                        ensure!(
                            self.read_unsigned_leb128(32)? == 0,
                            "invalid memory instruction"
                        );
                    }
                    0x11 => {
                        ensure!(
                            self.read_unsigned_leb128(32)? == 0,
                            "invalid memory instruction"
                        );
                    }

                    // table instructions
                    0x12 | 0x14 => {
                        self.read_unsigned_leb128(32)?;
                        self.read_unsigned_leb128(32)?;
                    }
                    0x13 | 0x15 | 0x16 | 0x17 => {
                        self.read_unsigned_leb128(32)?;
                    }
                    _ => bail!("invalid table instruction: {}", kind),
                }
            }

            _ => bail!("invalid opcode: {}", opcode),
        }

        Ok(())
    }
}
