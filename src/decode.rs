trait ReadExt: std::io::Read {
    fn read_u8(&mut self) -> std::io::Result<u8> {
        let mut a = [0u8; 1];
        self.read_exact(&mut a)?;
        Ok(a[0])
    }

    fn read_unsigned_leb128(&mut self, n: u64) -> std::io::Result<u64> {
        let a = self.read_u8()?;
        if a < 128 || a < (1 << 7) {
            Ok(a as u64)
        } else {
            let b = self.read_unsigned_leb128(n - 7)?;
            Ok(128 * b + (a as u64 - 128))
        }
    }
}
impl<R: std::io::Read + ?Sized> ReadExt for R {}

fn decode_section(buf: &mut impl std::io::BufRead) -> Result<(), Box<dyn std::error::Error>> {
    let idx = buf.read_unsigned_leb128(8)?;
    let size = buf.read_unsigned_leb128(32)?;

    assert!(idx <= 12, "invalid section id: {}", idx);

    let mut cont = vec![0u8; size as usize];
    buf.read_exact(cont.as_mut_slice())?;

    Ok(())
}

pub fn decode_module(buf: &mut impl std::io::BufRead) -> Result<(), Box<dyn std::error::Error>> {
    let mut header = [0u8; 8];
    buf.read_exact(&mut header)?;

    // magic number, version
    assert_eq!(header, [0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]);

    loop {
        decode_section(buf)?;
        if buf.fill_buf()?.is_empty() {
            break;
        }
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
