use super::{types::ReadTypeExt, value::ReadValueExt};
use anyhow::{bail, Result};
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
            _ => bail!("invalid opcode: {}", opcode),
        }

        Ok(())
    }
}
