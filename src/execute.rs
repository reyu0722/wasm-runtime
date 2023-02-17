use crate::core::{Func, FuncIdx, FuncType, Idx, Instruction, LocalIdx, Module, TypeIdx};
use anyhow::{bail, Result};
use std::{collections::VecDeque, rc::Rc};

pub struct Address<T> {
    pub address: u32,
    _phantom: std::marker::PhantomData<fn() -> T>,
}

impl<T> Address<T> {
    pub fn new(address: u32) -> Self {
        Self {
            address,
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct FuncAddr;

pub struct ModuleInstance {
    types: Vec<Rc<FuncType>>,
    func_addrs: Vec<Address<FuncAddr>>,
}

impl ModuleInstance {
    pub fn get_type(&self, idx: Idx<TypeIdx>) -> Rc<FuncType> {
        self.types[idx.get() as usize].clone()
    }
}

pub struct FuncInstance {
    ty: Rc<FuncType>,
    code: Func,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

pub enum StackEntry {
    Value(Value),
    // TODO: label
    Frame(Frame),
}

#[derive(Default)]
pub struct Stack {
    data: VecDeque<StackEntry>,
}

impl Stack {
    fn push_value(&mut self, value: Value) {
        self.data.push_front(StackEntry::Value(value));
    }

    fn pop_value(&mut self) -> Result<Value> {
        match self.data.pop_front() {
            Some(StackEntry::Value(value)) => Ok(value),
            _ => bail!("expected value on stack"),
        }
    }

    fn push_frame(&mut self, frame: Frame) {
        self.data.push_front(StackEntry::Frame(frame));
    }
}

#[derive(Default)]
pub struct Frame {
    locals: Vec<Value>,
}

impl Frame {
    pub fn get_local(&self, idx: Idx<LocalIdx>) -> Value {
        self.locals[idx.get() as usize]
    }
}

#[derive(Default)]
pub struct Store {
    funcs: Vec<FuncInstance>,
    stack: Stack,
}

impl Store {
    fn alloc_func(&mut self, func: Func, module: &ModuleInstance) -> Address<FuncAddr> {
        let ty = module.get_type(func.type_id);
        let i = FuncInstance { ty, code: func };
        self.funcs.push(i);
        Address::new(self.funcs.len() as u32 - 1)
    }

    fn alloc_module(&mut self, module: Module) -> ModuleInstance {
        let mut instance = ModuleInstance {
            types: module.types.into_iter().map(Rc::new).collect(),
            func_addrs: Vec::with_capacity(module.funcs.len()),
        };

        for func in module.funcs {
            let addr = self.alloc_func(func, &instance);
            instance.func_addrs.push(addr);
        }

        instance
    }

    pub fn instantiate(&mut self, module: Module) {
        let instance = self.alloc_module(module);
        let frame = Frame::default();
        self.stack.push_frame(frame);
    }

    fn execute(&mut self, idx: Idx<FuncIdx>) -> Result<()> {
        let func = &self.funcs[idx.get() as usize];
        for instr in &func.code.body.instructions {
            match instr {
                Instruction::I32Const(i) => {
                    self.stack.push_value(Value::I32(*i));
                }
                Instruction::I32Add => {
                    let v2 = self.stack.pop_value()?;
                    let v1 = self.stack.pop_value()?;
                    match (v1, v2) {
                        (Value::I32(v1), Value::I32(v2)) => {
                            self.stack.push_value(Value::I32(v1 + v2));
                        }
                        _ => bail!("expected i32 values"),
                    }
                }
                _ => unimplemented!(),
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Expression, Func, FuncType, Instruction, Module};
    use crate::decode::decode;

    fn execute_instructions(instructions: Vec<Instruction>) -> Result<Value> {
        let mut store = Store::default();
        let mut module = Module::default();
        let ty = FuncType {
            params: vec![],
            results: vec![],
        };
        let func = Func {
            type_id: Idx::new(0),
            locals: vec![],
            body: Expression { instructions },
        };

        module.types.push(ty);
        module.funcs.push(func);

        store.instantiate(module);
        store.execute(Idx::new(0))?;
        store.stack.pop_value()
    }

    #[test]
    fn test_execute() {
        let add = vec![
            Instruction::I32Const(1),
            Instruction::I32Const(2),
            Instruction::I32Add,
        ];
        let value = execute_instructions(add).unwrap();
        assert_eq!(value, Value::I32(3));
    }

    #[test]
    fn test_decode_and_exec() {
        let file = std::fs::File::open("test/test.wasm").unwrap();
        let mut reader = std::io::BufReader::new(file);
        let module = decode(&mut reader).unwrap();

        let mut store = Store::default();
        store.instantiate(module);
        store.execute(Idx::new(1)).unwrap();
        let value = store.stack.pop_value().unwrap();
        assert_eq!(value, Value::I32(42));
    }
}
