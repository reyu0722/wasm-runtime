use anyhow::{bail, ensure, Context as _, Result};
use std::io::{BufRead, Read};

mod types;
mod value;
use types::ReadTypeExt;
use value::ReadValueExt;

fn decode_type_section(buf: &mut impl Read) -> Result<()> {
    let size = buf
        .read_unsigned_leb128(32)
        .context("failed to read type section size")?;

    for _ in 0..size {
        buf.read_func_type()?;
    }

    Ok(())
}

fn decode_import_section(buf: &mut impl Read) -> Result<()> {
    let size = buf
        .read_unsigned_leb128(32)
        .context("failed to read import section size")?;

    for _ in 0..size {
        buf.read_name()?; // module
        buf.read_name()?; // name

        let desc = buf.read_u8().context("failed to read import desc")?;
        match desc {
            0x00 => {
                buf.read_unsigned_leb128(32)
                    .context("failed to read type id")?;
            }
            0x01 => {
                buf.read_table_type()?;
            }
            0x02 => {
                buf.read_limits()?;
            }
            0x03 => {
                buf.read_global_type()?;
            }

            _ => bail!("invalid import desc: {}", desc),
        }
    }

    Ok(())
}

fn decode_section(buf: &mut impl Read) -> Result<()> {
    let idx = buf
        .read_unsigned_leb128(8)
        .context("failed to read section index")?;
    let size = buf
        .read_unsigned_leb128(32)
        .context("failed to read section size")?;

    ensure!(idx <= 12, "invalid section id: {}", idx);

    let mut cont = vec![0u8; size as usize];
    buf.read_exact(cont.as_mut_slice())
        .context("failed to read section content")?;

    match idx {
        1 => decode_type_section(&mut std::io::Cursor::new(cont)),
        2 => decode_import_section(&mut std::io::Cursor::new(cont)),
        _ => Ok(()),
    }
}

pub fn decode_module(buf: &mut impl BufRead) -> Result<()> {
    let mut header = [0u8; 8];
    buf.read_exact(&mut header)?;

    ensure!(
        header[0..4] == [0x00, 0x61, 0x73, 0x6d],
        "invalid magic number"
    );
    ensure!(header[4..8] == [0x01, 0x00, 0x00, 0x00], "invalid version");

    loop {
        if !buf.has_data_left()? {
            break;
        }
        decode_section(buf)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::BufReader;

    #[test]
    fn test() {
        let f = File::open("test/add.wasm").unwrap();
        let mut buf = BufReader::new(f);
        decode_module(&mut buf).unwrap();
    }
}
