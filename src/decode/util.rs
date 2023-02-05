use anyhow::{anyhow, Result};
use std::io::BufRead;

pub trait ReadUtilExt: BufRead {
    fn read_byte(&mut self) -> Result<u8> {
        let mut a = [0u8; 1];
        self.read_exact(&mut a)?;
        Ok(a[0])
    }

    fn read_if_equal(&mut self, b: u8) -> Result<bool> {
        let top = self
            .fill_buf()?
            .first()
            .copied()
            .ok_or_else(|| anyhow!("unexpected EOF"))?;

        if top == b {
            self.consume(1);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn read_and_ensure(&mut self, b: u8) -> Result<u8> {
        let top = self
            .fill_buf()?
            .first()
            .copied()
            .ok_or_else(|| anyhow!("unexpected EOF"))?;

        if top == b {
            self.consume(1);
            Ok(top)
        } else {
            Err(anyhow!("expected {}, got {}", b, top))
        }
    }
}

impl<R: BufRead + ?Sized> ReadUtilExt for R {}

#[macro_export]
macro_rules! read_vec {
    ($r: expr, $x: expr) => {{
        let size = $r.read_u32().context("failed to read vec size")?;
        let mut vec = Vec::with_capacity(size as usize);
        for _ in 0..size {
            vec.push($x);
        }
        vec
    }};
}
