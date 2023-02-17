use self::prelude::*;
use crate::core::Module;
use anyhow::{ensure, Result};
use std::io::BufRead;

mod instruction;
mod prelude;
mod section;
mod types;
mod util;
mod value;

pub fn decode(buf: &mut impl BufRead) -> Result<Module> {
    let mut header = [0u8; 8];
    buf.read_exact(&mut header)?;

    ensure!(
        header[0..4] == [0x00, 0x61, 0x73, 0x6d],
        "invalid magic number"
    );
    ensure!(header[4..8] == [0x01, 0x00, 0x00, 0x00], "invalid version");

    let mut module = Module::default();

    while buf.has_data_left()? {
        buf.read_section(&mut module)?;
    }
    Ok(module)
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
        decode(&mut buf).unwrap();
    }
}
