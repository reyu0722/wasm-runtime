use super::prelude::*;
use crate::core::Name;
use anyhow::{bail, Context as _, Result};
use std::io::BufRead;

pub trait ReadValueExt: BufRead {
    fn read_unsigned_leb128(&mut self, n: u64) -> Result<u64> {
        let a = self.read_byte()?;
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
        let a = self.read_byte()?;
        if a < 64 && (n >= 7 || a < (1 << (n - 1))) {
            Ok(a as i64)
        } else if (64..128).contains(&a) && (n >= 8 || a >= (128 - (1 << (n - 1)))) {
            Ok(a as i64 - 128)
        } else if a >= 128 && n > 7 {
            let b = self.read_signed_leb128(n - 7)?;
            Ok(128 * b + (a as i64 - 128))
        } else {
            bail!("invalid signed leb128")
        }
    }

    fn read_u32(&mut self) -> Result<u32> {
        let val = self
            .read_unsigned_leb128(32)
            .context("failed to read u32")?;
        Ok(val as u32)
    }

    fn read_name(&mut self) -> Result<Name> {
        let size = self.read_u32().context("failed to read name size")?;
        let mut cont = vec![0u8; size as usize];
        self.read_exact(cont.as_mut_slice())
            .context("failed to read name content")?;
        Ok(Name::new(String::from_utf8(cont)?))
    }
}

impl<R: BufRead + ?Sized> ReadValueExt for R {}

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
