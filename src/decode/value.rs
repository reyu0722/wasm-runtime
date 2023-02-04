use anyhow::{bail, Context as _, Result};
use std::io::Read;

pub trait ReadValueExt: Read {
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
            bail!("invalid unsigned leb128")
        }
    }

    fn read_signed_leb128(&mut self, n: u64) -> Result<i64> {
        let a = self.read_u8()?;
        if a < 64 && (n >= 7 || a < (1 << (n - 1))) {
            Ok(a as i64)
        } else if a >= 64 && a < 128 && (n >= 8 || a > (128 - (1 << (n - 1)))) {
            Ok((a - 128) as i64)
        } else if a > 128 && n > 7 {
            let b = self.read_signed_leb128(n - 7)?;
            Ok(128 * b + (a as i64 - 128))
        } else {
            bail!("invalid signed leb128")
        }
    }

    fn read_name(&mut self) -> Result<()> {
        let size = self
            .read_unsigned_leb128(32)
            .context("failed to read name size")?;
        let mut cont = vec![0u8; size as usize];
        self.read_exact(cont.as_mut_slice())
            .context("failed to read name content")?;
        Ok(())
    }
}

impl<R: Read + ?Sized> ReadValueExt for R {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

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
}
