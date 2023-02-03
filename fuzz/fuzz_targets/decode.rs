#![no_main]

use std::io::BufReader;
use libfuzzer_sys::fuzz_target;
use wasm_smith::Module;
use wasm_runtime::decode::decode_module;

fuzz_target!(|module: Module| {
    let wasm_bytes = module.to_bytes();
    let mut reader = BufReader::new(wasm_bytes.as_slice());

    decode_module(&mut reader).unwrap();
});