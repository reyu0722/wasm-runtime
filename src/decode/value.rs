use anyhow::{bail, Context as _, Result};
use std::io::Read;

pub trait ReadExt: Read {
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
impl<R: std::io::Read + ?Sized> ReadExt for R {}
