use anyhow::{anyhow, Result};
use std::io::BufRead;

pub trait ReadUtilExt: BufRead {
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
