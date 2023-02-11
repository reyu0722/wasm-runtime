use super::prelude::*;
use crate::core::{
    Element, ElementMode, Export, ExportDesc, Expression, Func, FuncType, Global, Import,
    ImportDesc, MemoryType, Module, RefType, TableType,
};
use anyhow::{bail, ensure, Context as _, Result};
use std::io::{BufRead, Cursor};

pub trait ReadSectionExt: BufRead {
    fn read_section(&mut self, module: &mut Module) -> Result<()> {
        let idx = self
            .read_unsigned_leb128(8)
            .context("failed to read section index")?;
        let size = self.read_u32().context("failed to read section size")?;

        ensure!(idx <= 12, "invalid section id: {}", idx);

        let mut cont = vec![0u8; size as usize];
        self.read_exact(cont.as_mut_slice())
            .context("failed to read section content")?;
        let mut cursor = Cursor::new(cont);

        match idx {
            1 => {
                module.types = cursor.read_type_section()?;
            }
            2 => {
                module.imports = cursor.read_import_section()?;
            }
            3 => {
                module.funcs = cursor
                    .read_function_section()?
                    .into_iter()
                    .map(|id| Func {
                        type_id: id,
                        locals: vec![],
                        body: Expression {},
                    })
                    .collect();
            }
            4 => {
                module.tables = cursor.read_table_section()?;
            }
            5 => {
                module.memories = cursor.read_memory_section()?;
            }
            6 => {
                module.globals = cursor.read_global_section()?;
            }
            7 => {
                module.exports = cursor.read_export_section()?;
            }
            8 => {
                module.start = Some(cursor.read_start_section()?);
            }
            9 => {module.elements = cursor.read_element_section()?;},
            10 => cursor.read_code_section(module)?,
            11 => cursor.read_data_section()?,
            12 => cursor.read_data_count_section()?,
            _ => bail!("invalid section id: {}", idx),
        };

        ensure!(!cursor.has_data_left()?, "invalid section size");
        Ok(())
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

    fn read_start_section(&mut self) -> Result<u32> {
        let func_id = self
            .read_u32()
            .context("failed to read start section func id")?;

        Ok(func_id)
    }

    fn read_element_section(&mut self) -> Result<Vec<Element>> {
        let elements = read_vec!(self, {
            let ty = self.read_u32()?;

            match ty {
                0 => {
                    let offset = self.read_expr()?;
                    let _y = read_vec!(self, self.read_u32()?);

                    Element {
                        ty: RefType::Funcref,
                        init: Vec::new(),
                        mode: ElementMode::Active { table: 0, offset },
                    }
                }
                1 => {
                    self.read_and_ensure(0x00)?;
                    let _y = read_vec!(self, self.read_u32()?);

                    Element {
                        ty: RefType::Funcref,
                        init: Vec::new(),
                        mode: ElementMode::Passive,
                    }
                }
                2 => {
                    let table = self.read_u32()?;
                    let offset = self.read_expr()?;
                    self.read_and_ensure(0x00)?;
                    let _y = read_vec!(self, self.read_u32()?);

                    Element {
                        ty: RefType::Funcref,
                        init: Vec::new(),
                        mode: ElementMode::Active { table, offset },
                    }
                }
                3 => {
                    self.read_and_ensure(0x00)?;
                    let _y = read_vec!(self, self.read_u32()?);

                    Element {
                        ty: RefType::Funcref,
                        init: Vec::new(),
                        mode: ElementMode::Declarative,
                    }
                }
                4 => {
                    let offset = self.read_expr()?;
                    let _y = read_vec!(self, self.read_expr()?);

                    Element {
                        ty: RefType::Funcref,
                        init: Vec::new(),
                        mode: ElementMode::Active { table: 0, offset },
                    }
                }
                5 => {
                    let ref_type = self.read_byte()?;
                    let _y = read_vec!(self, self.read_expr()?);

                    Element {
                        ty: match ref_type {
                            0x70 => RefType::Funcref,
                            0x6f => RefType::Externref,
                            _ => bail!("invalid element section type"),
                        },
                        init: Vec::new(),
                        mode: ElementMode::Passive,
                    }
                }
                6 => {
                    let table = self.read_u32()?;
                    let offset = self.read_expr()?;
                    let ref_type = self.read_byte()?;
                    let _y = read_vec!(self, self.read_expr()?);

                    Element {
                        ty: match ref_type {
                            0x70 => RefType::Funcref,
                            0x6f => RefType::Externref,
                            _ => bail!("invalid element section type"),
                        },
                        init: Vec::new(),
                        mode: ElementMode::Active { table, offset },
                    }
                }
                7 => {
                    let ref_type = self.read_byte()?;
                    ensure!(
                        ref_type == 0x70 || ref_type == 0x6f,
                        "invalid element section type"
                    );
                    let _y = read_vec!(self, self.read_expr()?);

                    Element {
                        ty: match ref_type {
                            0x70 => RefType::Funcref,
                            0x6f => RefType::Externref,
                            _ => bail!("invalid element section type"),
                        },
                        init: Vec::new(),
                        mode: ElementMode::Declarative,
                    }
                }
                _ => bail!("invalid element section type: {}", ty),
            }
        });

        Ok(elements)
    }

    fn read_code_section(&mut self, module: &mut Module) -> Result<()> {
        let vec = read_vec!(self, {
            // TODO: check size
            self.read_u32()
                .context("failed to read code section body size")?;

            let types = read_vec!(self, {
                let n = self.read_u32()?;
                let ty = self.read_value_type()?;
                (n, ty)
            })
            .into_iter()
            .flat_map(|(n, ty)| std::iter::repeat(ty).take(n as usize))
            .collect();

            let expr = self.read_expr()?;
            (types, expr)
        });

        ensure!(
            vec.len() == module.funcs.len(),
            "code section size mismatch"
        );

        for (i, (locals, body)) in vec.into_iter().enumerate() {
            module.funcs[i].locals = locals;
            module.funcs[i].body = body;
        }

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
