use anyhow::{bail, ensure, Context as _, Result};
use std::io::{BufRead, Read};

trait ReadExt: Read {
    fn read_u8(&mut self) -> Result<u8> {
        let mut a = [0u8; 1];
        self.read_exact(&mut a)?;
        Ok(a[0])
    }

    fn read_unsigned_leb128(&mut self, n: u64) -> Result<u64> {
        let a = self.read_u8()?;
        if a < 128 && (n >= 7 || a < (1 << n)) {
            Ok(a as u64)
        } else if a >= 128 && n > 7 {
            let b = self.read_unsigned_leb128(n - 7)?;
            Ok(128 * b + (a as u64 - 128))
        } else {
            bail!("invalid leb128")
        }
    }
}
impl<R: std::io::Read + ?Sized> ReadExt for R {}

fn decode_result_type(buf: &mut impl BufRead) -> Result<()> {
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

fn decode_func_type(buf: &mut impl BufRead) -> Result<()> {
    let magic = buf.read_u8().context("failed to read magic number")?;
    ensure!(magic == 0x60, "invalid magic number: {}", magic);

    decode_result_type(buf)?;
    decode_result_type(buf)?;

    Ok(())
}

fn decode_type_section(buf: &mut impl BufRead) -> Result<()> {
    let size = buf
        .read_unsigned_leb128(32)
        .context("failed to read type section size")?;

    for _ in 0..size {
        decode_func_type(buf)?;
    }

    Ok(())
}

fn decode_section(buf: &mut impl BufRead) -> Result<()> {
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
    use std::io::{BufReader, Cursor};

    #[test]
    fn test_read_leb128() {
        fn lsb_from_buf_u8(buf: &[u8]) -> Result<u64> {
            let mut reader = Cursor::new(buf);
            reader.read_unsigned_leb128(8)
        }

        fn lsb_from_buf_u32(buf: &[u8]) -> Result<u64> {
            let mut reader = Cursor::new(buf);
            reader.read_unsigned_leb128(32)
        }

        assert_eq!(lsb_from_buf_u32(&[0x10]).unwrap(), 0x10);
        assert_eq!(lsb_from_buf_u32(&[0x80, 0x02]).unwrap(), 0x100);
        assert!(lsb_from_buf_u8(&[0x80]).is_err());
        assert!(lsb_from_buf_u8(&[0x80, 0x02]).is_err());
    }

    #[test]
    fn test() {
        let f = File::open("test/add.wasm").unwrap();
        let mut buf = BufReader::new(f);
        decode_module(&mut buf).unwrap();
    }
}
