use anyhow::Result;
use std::io::{Cursor, Read};
use wasm_runtime::{
    core::{Idx, Module},
    decode::decode,
    execute::{Store, Value},
};
use wast::{
    core::{WastArgCore, WastRetCore},
    parser::{parse, ParseBuffer},
    WastArg, WastDirective, WastExecute, WastInvoke, WastRet,
};

#[test]
fn test_i32() {
    let mut f = std::fs::File::open("tests/testsuite/i32.wast").unwrap();
    let mut b = vec![];
    f.read_to_end(&mut b).unwrap();
    let s = std::str::from_utf8(&b).unwrap();

    let buf = ParseBuffer::new(s).unwrap();
    let wast = parse::<wast::Wast>(&buf).unwrap();
    let mut module = Module::default();

    for dir in wast.directives {
        match dir {
            WastDirective::Wat(mut wat) => {
                let bin = wat.encode().unwrap();
                module = decode(&mut Cursor::new(bin)).unwrap();
            }
            WastDirective::AssertReturn { exec, results, .. } => {
                let mut store = Store::default();
                store.instantiate(module.clone());

                let (name, args) = match exec {
                    WastExecute::Invoke(WastInvoke { name, args, .. }) => (name, args),
                    _ => unimplemented!(),
                };

                println!("name: {}, args: {:?}", name, args);

                let args = args
                    .into_iter()
                    .map(|a| wast_arg_to_value(a).unwrap())
                    .collect::<Vec<_>>();

                let expected = results
                    .into_iter()
                    .map(|r| wast_ret_to_value(r).unwrap())
                    .collect::<Vec<_>>();

                let actual = store.invoke(name, args).unwrap();

                assert_eq!(actual, expected);
            }
            _ => {}
        }
    }
}

fn wast_arg_to_value(arg: WastArg) -> Result<Value> {
    match arg {
        WastArg::Core(WastArgCore::I32(v)) => Ok(Value::I32(v)),
        WastArg::Core(WastArgCore::I64(v)) => Ok(Value::I64(v)),
        _ => unimplemented!(),
    }
}

fn wast_ret_to_value(ret: WastRet) -> Result<Value> {
    match ret {
        WastRet::Core(WastRetCore::I32(v)) => Ok(Value::I32(v)),
        WastRet::Core(WastRetCore::I64(v)) => Ok(Value::I64(v)),
        _ => unimplemented!(),
    }
}
