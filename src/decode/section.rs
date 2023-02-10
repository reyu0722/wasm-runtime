use super::prelude::*;
use crate::core::{
    Export, ExportDesc, FuncType, Global, Import, ImportDesc, MemoryType, Module, TableType,
};
use anyhow::{bail, ensure, Context as _, Result};
use std::io::{BufRead, Cursor};

pub trait ReadSectionExt: BufRead {
    fn read_section(&mut self) -> Result<Module> {
        let idx = self
            .read_unsigned_leb128(8)
            .context("failed to read section index")?;
        let size = self.read_u32().context("failed to read section size")?;

        ensure!(idx <= 12, "invalid section id: {}", idx);

        let mut cont = vec![0u8; size as usize];
        self.read_exact(cont.as_mut_slice())
            .context("failed to read section content")?;
        let mut cursor = Cursor::new(cont);

        let mut types = Vec::new();
        let mut imports = Vec::new();
        let mut _funcs_idx = Vec::new();
        let mut tables = Vec::new();
        let mut memories = Vec::new();
        let mut globals = Vec::new();
        let mut exports = Vec::new();

        match idx {
            1 => {
                types = cursor.read_type_section()?;
            }
            2 => {
                imports = cursor.read_import_section()?;
            }
            3 => {
                _funcs_idx = cursor.read_function_section()?;
            }
            4 => {
                tables = cursor.read_table_section()?;
            }
            5 => {
                memories = cursor.read_memory_section()?;
            }
            6 => {
                globals = cursor.read_global_section()?;
            }
            7 => {
                exports = cursor.read_export_section()?;
            }
            8 => cursor.read_start_section()?,
            9 => cursor.read_element_section()?,
            10 => cursor.read_code_section()?,
            11 => cursor.read_data_section()?,
            12 => cursor.read_data_count_section()?,
            _ => bail!("invalid section id: {}", idx),
        };

        ensure!(!cursor.has_data_left()?, "invalid section size");
        Ok(Module {
            types,
            imports,
            exports,
            tables,
            memories,
            globals,
        })
    }

    fn read_type_section(&mut self) -> Result<Vec<FuncType>> {
        Ok(read_vec!(self, self.read_func_type()?))
    }

    fn read_import_section(&mut self) -> Result<Vec<Import>> {
        let vec = read_vec!(self, {
            let module = self.read_name()?; // module
            let name = self.read_name()?; // name

            let desc_type = self.read_byte().context("failed to read import desc")?;
            let desc = match desc_type {
                0x00 => {
                    let func = self.read_u32().context("failed to read type id")?;
                    ImportDesc::Func(func)
                }
                0x01 => {
                    let table = self.read_table_type()?;
                    ImportDesc::Table(table)
                }
                0x02 => {
                    let limits = self.read_limits()?;
                    ImportDesc::Memory(limits)
                }
                0x03 => {
                    let global_type = self.read_global_type()?;
                    ImportDesc::Global(global_type)
                }
                _ => bail!("invalid import desc: {}", desc_type),
            };

            Import { module, name, desc }
        });

        Ok(vec)
    }

    fn read_function_section(&mut self) -> Result<Vec<u32>> {
        let vec = read_vec!(self, self.read_u32()?);
        Ok(vec)
    }

    fn read_table_section(&mut self) -> Result<Vec<TableType>> {
        let table_type = read_vec!(self, self.read_table_type()?);
        Ok(table_type)
    }

    fn read_memory_section(&mut self) -> Result<Vec<MemoryType>> {
        let memories = read_vec!(self, self.read_limits()?);
        Ok(memories)
    }

    fn read_global_section(&mut self) -> Result<Vec<Global>> {
        let vec = read_vec!(self, {
            Global {
                global_type: self.read_global_type()?,
                init: self.read_expr()?,
            }
        });
        Ok(vec)
    }

    fn read_export_section(&mut self) -> Result<Vec<Export>> {
        let vec = read_vec!(self, {
            let name = self.read_name()?;
            let ty = self.read_byte().context("failed to read export type")?;
            let id = self.read_u32().context("failed to read export id")?;

            let desc = match ty {
                0x00 => ExportDesc::Func(id),
                0x01 => ExportDesc::Table(id),
                0x02 => ExportDesc::Memory(id),
                0x03 => ExportDesc::Global(id),
                _ => bail!("invalid export type: {}", ty),
            };

            Export { name, desc }
        });
        Ok(vec)
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
