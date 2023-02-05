use super::prelude::*;
use anyhow::{bail, ensure, Context as _, Result};
use std::io::BufRead;

pub trait ReadTypeExt: BufRead {
    fn read_value_type(&mut self) -> Result<()> {
        let ty = self.read_byte().context("failed to read value type")?;

        match ty {
            0x7f => (), // i32
            0x7e => (), // i64
            0x7d => (), // u32
            0x7c => (), // u64
            0x7b => (), // v128
            0x70 => (), // funcref
            0x6f => (), // externref
            _ => bail!("invalid value type: {}", ty),
        }

        Ok(())
    }

    fn read_result_type(&mut self) -> Result<()> {
        read_vec!(
            self,
            self.read_value_type().context("failed to read result type")
        );

        Ok(())
    }

    fn read_func_type(&mut self) -> Result<()> {
        let magic = self.read_byte().context("failed to read magic number")?;
        ensure!(magic == 0x60, "invalid magic number: {}", magic);

        self.read_result_type()?;
        self.read_result_type()?;

        Ok(())
    }

    fn read_limits(&mut self) -> Result<()> {
        let flag = self.read_byte().context("failed to read limits flag")?;
        match flag {
            0x00 => {
                self.read_u32().context("failed to read limits min")?;
            }
            0x01 => {
                self.read_u32().context("failed to read limits min")?;
                self.read_u32().context("failed to read limits max")?;
            }
            _ => bail!("invalid limits flag: {}", flag),
        }

        Ok(())
    }

    fn read_table_type(&mut self) -> Result<()> {
        let reftype = self.read_byte().context("failed to read reftype")?;
        ensure!(
            reftype == 0x70 || reftype == 0x6f,
            "invalid reftype: {}",
            reftype
        );

        self.read_limits()?;
        Ok(())
    }

    fn read_global_type(&mut self) -> Result<()> {
        let valtype = self.read_byte().context("failed to read valtype")?;
        ensure!(
            valtype == 0x7f || valtype == 0x7e || valtype == 0x7d || valtype == 0x7c,
            "invalid valtype: {}",
            valtype
        );

        let mutability = self.read_byte().context("failed to read mutability")?;
        ensure!(mutability <= 1, "invalid mutability: {}", mutability);

        Ok(())
    }
}

impl<R: BufRead + ?Sized> ReadTypeExt for R {}
