#![no_main]

use std::io::BufReader;
use libfuzzer_sys::fuzz_target;
use wasm_smith::Module;
use wasm_runtime::decode::decode;

fuzz_target!(|module: Module| {
    let wasm_bytes = module.to_bytes();
    let mut reader = BufReader::new(wasm_bytes.as_slice());

    decode(&mut reader).unwrap();
});