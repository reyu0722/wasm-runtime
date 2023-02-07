use super::prelude::*;
use crate::core::{
    FuncType, GlobalType, Limits, NumType, RefType, ResultType, TableType, ValueType, VecType,
};
use anyhow::{bail, ensure, Context as _, Result};
use std::io::BufRead;

pub trait ReadTypeExt: BufRead {
    fn read_value_type(&mut self) -> Result<ValueType> {
        let ty = self.read_byte().context("failed to read value type")?;

        let res = match ty {
            0x7f => ValueType::Num(NumType::I32),       // i32
            0x7e => ValueType::Num(NumType::I64),       // i64
            0x7d => ValueType::Num(NumType::F32),       // f32
            0x7c => ValueType::Num(NumType::F64),       // f64
            0x7b => ValueType::Vec(VecType::V128),      // v128
            0x70 => ValueType::Ref(RefType::Funcref),   // funcref
            0x6f => ValueType::Ref(RefType::Externref), // externref
            _ => bail!("invalid value type: {}", ty),
        };

        Ok(res)
    }

    fn read_result_type(&mut self) -> Result<ResultType> {
        let vec = read_vec!(
            self,
            self.read_value_type()
                .context("failed to read result type")?
        );

        Ok(vec)
    }

    fn read_func_type(&mut self) -> Result<FuncType> {
        self.read_and_ensure(0x60)
            .context("failed to read func type")?;

        Ok(FuncType {
            params: self.read_result_type()?,
            results: self.read_result_type()?,
        })
    }

    fn read_limits(&mut self) -> Result<Limits> {
        let flag = self.read_byte().context("failed to read limits flag")?;
        let limits = match flag {
            0x00 => {
                let min = self.read_u32().context("failed to read limits min")?;
                Limits { min, max: None }
            }
            0x01 => {
                let min = self.read_u32().context("failed to read limits min")?;
                let max = self.read_u32().context("failed to read limits max")?;
                Limits {
                    min,
                    max: Some(max),
                }
            }
            _ => bail!("invalid limits flag: {}", flag),
        };

        Ok(limits)
    }

    fn read_table_type(&mut self) -> Result<TableType> {
        let b = self.read_byte().context("failed to read reftype")?;
        let reftype = RefType::from_byte(b)?;

        let limits = self.read_limits()?;
        Ok(TableType {
            limits,
            elem_type: reftype,
        })
    }

    fn read_global_type(&mut self) -> Result<GlobalType> {
        let b = self.read_byte().context("failed to read valtype")?;
        let value_type = ValueType::from_byte(b)?;

        let mutability = self.read_byte().context("failed to read mutability")?;
        ensure!(mutability <= 1, "invalid mutability: {}", mutability);

        Ok(GlobalType {
            value_type,
            mutability: mutability == 1,
        })
    }
}

impl<R: BufRead + ?Sized> ReadTypeExt for R {}
