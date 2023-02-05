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
            7 => cursor.read_export_section(),
            8 => cursor.read_start_section(),
            9 => cursor.read_element_section(),
            10 => cursor.read_code_section(),
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

    fn read_export_section(&mut self) -> Result<()> {
        let size = self
            .read_unsigned_leb128(32)
            .context("failed to read export section size")?;

        for _ in 0..size {
            self.read_name()?;
            let ty = self.read_u8().context("failed to read export type")?;
            ensure!(ty <= 0x03, "invalid export type: {}", ty);

            self.read_unsigned_leb128(32)
                .context("failed to read export id")?;
        }

        Ok(())
    }

    fn read_start_section(&mut self) -> Result<()> {
        self.read_unsigned_leb128(32)
            .context("failed to read start section func id")?;

        Ok(())
    }

    fn read_element_section(&mut self) -> Result<()> {
        let size = self
            .read_unsigned_leb128(32)
            .context("failed to read element section size")?;

        for _ in 0..size {
            let ty = self.read_unsigned_leb128(32)?;

            match ty {
                0 => {
                    self.read_expr()?;
                    let size = self.read_unsigned_leb128(32)?;
                    for _ in 0..size {
                        self.read_unsigned_leb128(32)?;
                    }
                }
                1 => {
                    ensure!(self.read_u8()? == 0x00, "invalid element section type");
                    let size = self.read_unsigned_leb128(32)?;
                    for _ in 0..size {
                        self.read_unsigned_leb128(32)?;
                    }
                }
                2 => {
                    self.read_unsigned_leb128(32)?;
                    self.read_expr()?;
                    ensure!(self.read_u8()? == 0x00, "invalid element section type");
                    let size = self.read_unsigned_leb128(32)?;
                    for _ in 0..size {
                        self.read_unsigned_leb128(32)?;
                    }
                }
                3 => {
                    ensure!(self.read_u8()? == 0x00, "invalid element section type");
                    let size = self.read_unsigned_leb128(32)?;
                    for _ in 0..size {
                        self.read_unsigned_leb128(32)?;
                    }
                }
                4 => {
                    self.read_expr()?;
                    let size = self.read_unsigned_leb128(32)?;
                    for _ in 0..size {
                        self.read_expr()?;
                    }
                }
                5 => {
                    let ref_type = self.read_u8()?;
                    ensure!(
                        ref_type == 0x70 || ref_type == 0x6f,
                        "invalid element section type"
                    );
                    let size = self.read_unsigned_leb128(32)?;
                    for _ in 0..size {
                        self.read_expr()?;
                    }
                }
                6 => {
                    self.read_unsigned_leb128(32)?;
                    self.read_expr()?;
                    let ref_type = self.read_u8()?;
                    ensure!(
                        ref_type == 0x70 || ref_type == 0x6f,
                        "invalid element section type"
                    );
                    let size = self.read_unsigned_leb128(32)?;
                    for _ in 0..size {
                        self.read_expr()?;
                    }
                }
                7 => {
                    let ref_type = self.read_u8()?;
                    ensure!(
                        ref_type == 0x70 || ref_type == 0x6f,
                        "invalid element section type"
                    );
                    let size = self.read_unsigned_leb128(32)?;
                    for _ in 0..size {
                        self.read_expr()?;
                    }
                }
                _ => bail!("invalid element section type: {}", ty),
            }
        }

        Ok(())
    }

    fn read_code_section(&mut self) -> Result<()> {
        let size = self
            .read_unsigned_leb128(32)
            .context("failed to read code section size")?;

        for _ in 0..size {
            // TODO: check size
            self.read_unsigned_leb128(32)
                .context("failed to read code section body size")?;

            let size = self
                .read_unsigned_leb128(32)
                .context("failed to read code section vec size")?;
            for _ in 0..size {
                self.read_unsigned_leb128(32)?;
                self.read_value_type()?;
            }

            self.read_expr()?;
        }

        Ok(())
    }
}

impl<R: BufRead + ?Sized> ReadSectionExt for R {}
