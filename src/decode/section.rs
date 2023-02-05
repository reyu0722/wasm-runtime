use super::prelude::*;
use anyhow::{bail, ensure, Context as _, Result};
use std::io::{BufRead, Cursor};

pub trait ReadSectionExt: BufRead {
    fn read_section(&mut self) -> Result<()> {
        let idx = self
            .read_unsigned_leb128(8)
            .context("failed to read section index")?;
        let size = self.read_u32().context("failed to read section size")?;

        ensure!(idx <= 12, "invalid section id: {}", idx);

        let mut cont = vec![0u8; size as usize];
        self.read_exact(cont.as_mut_slice())
            .context("failed to read section content")?;
        let mut cursor = Cursor::new(cont);

        let res = match idx {
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
            11 => cursor.read_data_section(),
            12 => cursor.read_data_count_section(),
            _ => bail!("invalid section id: {}", idx),
        };

        ensure!(!cursor.has_data_left()?, "invalid section size");
        res
    }

    fn read_type_section(&mut self) -> Result<()> {
        read_vec!(self, self.read_func_type()?);
        Ok(())
    }

    fn read_import_section(&mut self) -> Result<()> {
        let size = self
            .read_u32()
            .context("failed to read import section size")?;

        for _ in 0..size {
            self.read_name()?; // module
            self.read_name()?; // name

            let desc = self.read_byte().context("failed to read import desc")?;
            match desc {
                0x00 => {
                    self.read_u32().context("failed to read type id")?;
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
        read_vec!(self, self.read_u32()?);
        Ok(())
    }

    fn read_table_section(&mut self) -> Result<()> {
        read_vec!(self, self.read_table_type()?);
        Ok(())
    }

    fn read_memory_section(&mut self) -> Result<()> {
        read_vec!(self, self.read_limits()?);
        Ok(())
    }

    fn read_global_section(&mut self) -> Result<()> {
        read_vec!(self, {
            self.read_global_type()?;
            self.read_expr()?
        });
        Ok(())
    }

    fn read_export_section(&mut self) -> Result<()> {
        read_vec!(self, {
            self.read_name()?;
            let ty = self.read_byte().context("failed to read export type")?;
            ensure!(ty <= 0x03, "invalid export type: {}", ty);

            self.read_u32().context("failed to read export id")?
        });
        Ok(())
    }

    fn read_start_section(&mut self) -> Result<()> {
        self.read_u32()
            .context("failed to read start section func id")?;

        Ok(())
    }

    fn read_element_section(&mut self) -> Result<()> {
        read_vec!(self, {
            let ty = self.read_u32()?;

            match ty {
                0 => {
                    self.read_expr()?;
                    read_vec!(self, self.read_u32()?);
                }
                1 => {
                    self.read_and_ensure(0x00)?;
                    read_vec!(self, self.read_u32()?);
                }
                2 => {
                    self.read_u32()?;
                    self.read_expr()?;
                    self.read_and_ensure(0x00)?;
                    read_vec!(self, self.read_u32()?);
                }
                3 => {
                    self.read_and_ensure(0x00)?;
                    read_vec!(self, self.read_u32()?);
                }
                4 => {
                    self.read_expr()?;
                    read_vec!(self, self.read_expr()?);
                }
                5 => {
                    let ref_type = self.read_byte()?;
                    ensure!(
                        ref_type == 0x70 || ref_type == 0x6f,
                        "invalid element section type"
                    );
                    read_vec!(self, self.read_expr()?);
                }
                6 => {
                    self.read_u32()?;
                    self.read_expr()?;
                    let ref_type = self.read_byte()?;
                    ensure!(
                        ref_type == 0x70 || ref_type == 0x6f,
                        "invalid element section type"
                    );
                    read_vec!(self, self.read_expr()?);
                }
                7 => {
                    let ref_type = self.read_byte()?;
                    ensure!(
                        ref_type == 0x70 || ref_type == 0x6f,
                        "invalid element section type"
                    );
                    read_vec!(self, self.read_expr()?);
                }
                _ => bail!("invalid element section type: {}", ty),
            }
        });

        Ok(())
    }

    fn read_code_section(&mut self) -> Result<()> {
        read_vec!(self, {
            // TODO: check size
            self.read_u32()
                .context("failed to read code section body size")?;

            read_vec!(self, {
                self.read_u32()?;
                self.read_value_type()?;
            });

            self.read_expr()?
        });

        Ok(())
    }

    fn read_data_section(&mut self) -> Result<()> {
        read_vec!(self, {
            let ty = self.read_u32()?;

            match ty {
                0 => {
                    self.read_expr()?;
                    read_vec!(self, self.read_byte()?);
                }
                1 => {
                    read_vec!(self, self.read_byte()?);
                }
                2 => {
                    self.read_u32()?;
                    self.read_expr()?;
                    read_vec!(self, self.read_byte()?);
                }
                _ => bail!("invalid data section type: {}", ty),
            }
        });

        Ok(())
    }

    fn read_data_count_section(&mut self) -> Result<()> {
        self.read_u32()
            .context("failed to read data count section size")?;

        Ok(())
    }
}

impl<R: BufRead + ?Sized> ReadSectionExt for R {}
