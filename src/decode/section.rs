use super::types::ReadTypeExt;
use super::value::ReadValueExt;
use anyhow::{bail, ensure, Context as _, Result};
use std::io::{Cursor, Read};

pub trait ReadSectionExt: Read {
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
}

impl<R: Read + ?Sized> ReadSectionExt for R {}
