use super::value::ReadExt;
use anyhow::{bail, ensure, Context as _, Result};
use std::io::Read;

fn decode_result_type(buf: &mut impl Read) -> Result<()> {
    let size = buf
        .read_unsigned_leb128(32)
        .context("failed to read result type size")?;

    for _ in 0..size {
        let ty = buf.read_u8().context("failed to read result type")?;

        match ty {
            0x7f => (), // i32
            0x7e => (), // i64
            0x7d => (), // u32
            0x7c => (), // u64
            0x7b => (), // v128
            0x70 => (), // funcref
            0x6f => (), // externref
            _ => bail!("invalid result type: {}", ty),
        }
    }

    Ok(())
}

pub fn decode_func_type(buf: &mut impl Read) -> Result<()> {
    let magic = buf.read_u8().context("failed to read magic number")?;
    ensure!(magic == 0x60, "invalid magic number: {}", magic);

    decode_result_type(buf)?;
    decode_result_type(buf)?;

    Ok(())
}

pub fn decode_limits(buf: &mut impl Read) -> Result<()> {
    let flag = buf.read_u8().context("failed to read limits flag")?;
    match flag {
        0x00 => {
            buf.read_unsigned_leb128(32)
                .context("failed to read limits min")?;
        }
        0x01 => {
            buf.read_unsigned_leb128(32)
                .context("failed to read limits min")?;
            buf.read_unsigned_leb128(32)
                .context("failed to read limits max")?;
        }
        _ => bail!("invalid limits flag: {}", flag),
    }

    Ok(())
}

pub fn decode_table_type(buf: &mut impl Read) -> Result<()> {
    let reftype = buf.read_u8().context("failed to read reftype")?;
    ensure!(
        reftype == 0x70 || reftype == 0x6f,
        "invalid reftype: {}",
        reftype
    );

    decode_limits(buf)?;
    Ok(())
}

pub fn decode_global_type(buf: &mut impl Read) -> Result<()> {
    let valtype = buf.read_u8().context("failed to read valtype")?;
    ensure!(
        valtype == 0x7f || valtype == 0x7e || valtype == 0x7d || valtype == 0x7c,
        "invalid valtype: {}",
        valtype
    );

    let mutability = buf.read_u8().context("failed to read mutability")?;
    ensure!(mutability <= 1, "invalid mutability: {}", mutability);

    Ok(())
}
