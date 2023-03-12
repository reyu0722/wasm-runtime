use paste::paste;
use std::fs::File;
use std::io::{Cursor, Read};
use wasm_runtime::{
    core::Module,
    decode::decode,
    execute::{Store, Value},
};
use wast::{
    core::{WastArgCore, WastRetCore},
    parser::{parse, ParseBuffer},
    WastArg, WastDirective, WastExecute, WastInvoke, WastRet,
};

testsuite!(i32);

#[macro_export]
macro_rules! testsuite {
    ($($f: ident), *) => {
        $(paste! {
            #[test]
            fn [< testsuite_ $f >]() {
                test_wast(format!("tests/testsuite/{}.wast", stringify!($f)).as_str());
            }
        })*
    };
}

fn test_wast(filename: &str) {
    let mut f = File::open(filename).unwrap();
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

                let WastExecute::Invoke(WastInvoke { name, args, .. }) = exec else {
                                                        unimplemented!()
                                                    };
                println!("name: {}, args: {:?}", name, args);

                let args = args.into_iter().map(wast_arg_to_value).collect::<Vec<_>>();

                let expected = results
                    .into_iter()
                    .map(wast_ret_to_value)
                    .collect::<Vec<_>>();

                let actual = store.invoke(name, args).unwrap();

                assert_eq!(actual, expected);
            }
            _ => {}
        }
    }
}

fn wast_arg_to_value(arg: WastArg) -> Value {
    match arg {
        WastArg::Core(WastArgCore::I32(v)) => Value::I32(v),
        WastArg::Core(WastArgCore::I64(v)) => Value::I64(v),
        _ => unimplemented!(),
    }
}

fn wast_ret_to_value(ret: WastRet) -> Value {
    match ret {
        WastRet::Core(WastRetCore::I32(v)) => Value::I32(v),
        WastRet::Core(WastRetCore::I64(v)) => Value::I64(v),
        _ => unimplemented!(),
    }
}
