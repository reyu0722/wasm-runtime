use super::instruction::ReadInstructionExt;
use super::types::ReadTypeExt;
use super::value::ReadValueExt;
use anyhow::{bail, ensure, Context as _, Result};
use std::io::{BufRead, Cursor};

pub trait ReadSectionExt: BufRead {
    fn read_section(&mut self) -> Result<()> {
        let idx = self
            .read_unsigned_leb128(8)
            .context("failed to read section index")?;
        let size = self
            .read_unsigned_leb128(32)
            .context("failed to read section size")?;

        ensure!(idx <= 12, "invalid section id: {}", idx);

        let mut cont = vec![0u8; size as usize];
        self.read_exact(cont.as_mut_slice())
            .context("failed to read section content")?;
        let mut cursor = Cursor::new(cont);

        match idx {
            1 => cursor.read_type_section(),
            2 => cursor.read_import_section(),
            3 => cursor.read_function_section(),
            4 => cursor.read_table_section(),
            5 => cursor.read_memory_section(),
            6 => cursor.read_global_section(),
            _ => Ok(()),
        }
    }

    fn read_type_section(&mut self) -> Result<()> {
        let size = self
            .read_unsigned_leb128(32)
            .context("failed to read type section size")?;

        for _ in 0..size {
            self.read_func_type()?;
        }

        Ok(())
    }

    fn read_import_section(&mut self) -> Result<()> {
        let size = self
            .read_unsigned_leb128(32)
            .context("failed to read import section size")?;

        for _ in 0..size {
            self.read_name()?; // module
            self.read_name()?; // name

            let desc = self.read_u8().context("failed to read import desc")?;
            match desc {
                0x00 => {
                    self.read_unsigned_leb128(32)
                        .context("failed to read type id")?;
                }
                0x01 => {
                    self.read_table_type()?;
                }
                0x02 => {
                    self.read_limits()?;
                }
                0x03 => {
                    self.read_global_type()?;
                }
                _ => bail!("invalid import desc: {}", desc),
            }
        }

        Ok(())
    }

    fn read_function_section(&mut self) -> Result<()> {
        let size = self
            .read_unsigned_leb128(32)
            .context("failed to read function section size")?;

        for _ in 0..size {
            self.read_unsigned_leb128(32)
                .context("failed to read type id")?;
        }

        Ok(())
    }

    fn read_table_section(&mut self) -> Result<()> {
        let size = self
            .read_unsigned_leb128(32)
            .context("failed to read table section size")?;

        for _ in 0..size {
            self.read_table_type()?;
        }

        Ok(())
    }

    fn read_memory_section(&mut self) -> Result<()> {
        let size = self
            .read_unsigned_leb128(32)
            .context("failed to read memory section size")?;

        for _ in 0..size {
            self.read_limits()?;
        }

        Ok(())
    }

    fn read_global_section(&mut self) -> Result<()> {
        let size = self
            .read_unsigned_leb128(32)
            .context("failed to read global section size")?;

        for _ in 0..size {
            self.read_global_type()?;
            self.read_expr()?;
        }

        Ok(())
    }
}

impl<R: BufRead + ?Sized> ReadSectionExt for R {}
