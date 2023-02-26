use crate::core::{
    BlockType, Func, FuncIdx, FuncType, Idx, Instruction, LocalIdx, Module, NumType, TypeIdx,
    ValueType,
};
use anyhow::{bail, ensure, Result};
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

impl Value {
    pub fn get_type(&self) -> ValueType {
        match self {
            Value::I32(_) => ValueType::Num(NumType::I32),
            Value::I64(_) => ValueType::Num(NumType::I64),
            Value::F32(_) => ValueType::Num(NumType::F32),
            Value::F64(_) => ValueType::Num(NumType::F64),
        }
    }
}

pub enum StackEntry<'a> {
    Value(Value),
    Label(Label<'a>),
    Frame(Frame),
}

pub struct Label<'a> {
    arity: usize,
    instr: &'a Vec<Instruction>,
}

#[derive(Default)]
pub struct Stack<'a> {
    data: VecDeque<StackEntry<'a>>,
    current_frame: Option<Frame>,
}

impl<'a> Stack<'a> {
    fn push_value(&mut self, value: Value) {
        self.data.push_front(StackEntry::Value(value));
    }

    fn push_frame(&mut self, frame: Frame) {
        self.data.push_front(StackEntry::Frame(frame));
    }

    fn push_label(&mut self, label: Label<'a>) {
        self.data.push_front(StackEntry::Label(label));
    }

    fn pop(&mut self) -> Option<StackEntry> {
        self.data.pop_front()
    }

    fn pop_value(&mut self) -> Result<Value> {
        match self.data.pop_front() {
            Some(StackEntry::Value(value)) => Ok(value),
            _ => bail!("expected value on stack"),
        }
    }

    fn pop_label(&mut self) -> Result<Label<'a>> {
        match self.data.pop_front() {
            Some(StackEntry::Label(label)) => Ok(label),
            _ => bail!("expected label on stack"),
        }
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
    }

    pub fn execute(&self, idx: Idx<FuncIdx>, args: Vec<Value>) -> Result<Vec<Value>> {
        let mut stack = Stack::default();
        self.execute_func(&mut stack, idx, args)
    }

    fn execute_func<'a>(
        &'a self,
        stack: &mut Stack<'a>,
        idx: Idx<FuncIdx>,
        locals: Vec<Value>,
    ) -> Result<Vec<Value>> {
        let func = &self.funcs[idx.get() as usize];

        let frame = Frame { locals };
        self.execute_label(stack, &frame, &func.code.body.instructions)?;

        let mut values = vec![];
        for ty in func.ty.results.iter() {
            let v = stack.pop_value()?;
            ensure!(v.get_type() == *ty, "type mismatch");
            values.push(v);
        }
        ensure!(stack.data.is_empty(), "stack is not empty");
        Ok(values)
    }

    fn execute_label<'a>(
        &self,
        stack: &mut Stack<'a>,
        frame: &Frame,
        instructions: &'a Vec<Instruction>,
    ) -> Result<()> {
        for instr in instructions {
            match instr {
                Instruction::Block {
                    block_type,
                    instructions,
                } => {
                    let iter: Vec<&ValueType> = match block_type {
                        BlockType::ValType(op) => op.iter().collect(),
                        _ => unimplemented!(),
                    };
                    let label = Label {
                        arity: iter.len(),
                        instr: instructions,
                    };
                    stack.push_label(label);

                    self.execute_label(stack, frame, instructions)?;

                    let mut values = vec![];
                    for ty in iter {
                        let v = stack.pop_value()?;
                        ensure!(v.get_type() == *ty, "type mismatch");
                        values.push(v);
                    }

                    stack.pop_label()?;
                    for v in values {
                        stack.push_value(v);
                    }
                }

                Instruction::LocalGet(idx) => {
                    let v = frame.get_local(*idx);
                    stack.push_value(v);
                }

                Instruction::I32Const(i) => {
                    stack.push_value(Value::I32(*i));
                }
                Instruction::I32Add => {
                    let v2 = stack.pop_value()?;
                    let v1 = stack.pop_value()?;
                    match (v1, v2) {
                        (Value::I32(v1), Value::I32(v2)) => {
                            stack.push_value(Value::I32(v1 + v2));
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
    use crate::core::{BlockType, Expression, Func, FuncType, Instruction, Module};
    use crate::decode::decode;

    fn execute_instructions(
        types: Vec<FuncType>,
        instructions: Vec<Instruction>,
        args: Vec<Value>,
    ) -> Result<Vec<Value>> {
        let mut store = Store::default();
        let mut module = Module::default();
        let func = Func {
            type_id: Idx::new(0),
            locals: vec![],
            body: Expression { instructions },
        };

        module.types = types;
        module.funcs.push(func);

        store.instantiate(module);
        store.execute(Idx::new(0), args)
    }

    #[test]
    fn test_add() {
        let types = vec![FuncType {
            params: vec![],
            results: vec![ValueType::Num(NumType::I32)],
        }];
        let add = vec![
            Instruction::I32Const(1),
            Instruction::I32Const(2),
            Instruction::I32Add,
        ];
        let value = execute_instructions(types, add, vec![]).unwrap();
        assert_eq!(value, vec![Value::I32(3)]);
    }

    #[test]
    fn test_add_with_args() {
        let types = vec![FuncType {
            params: vec![ValueType::Num(NumType::I32), ValueType::Num(NumType::I32)],
            results: vec![ValueType::Num(NumType::I32)],
        }];
        let add = vec![
            Instruction::LocalGet(Idx::from(0)),
            Instruction::LocalGet(Idx::from(1)),
            Instruction::I32Add,
        ];
        let value = execute_instructions(types, add, vec![Value::I32(4), Value::I32(7)]).unwrap();
        assert_eq!(value, vec![Value::I32(11)]);
    }

    #[test]
    fn test_block() {
        let types = vec![FuncType {
            params: vec![],
            results: vec![ValueType::Num(NumType::I32)],
        }];
        let block = vec![Instruction::Block {
            block_type: BlockType::ValType(Some(ValueType::Num(NumType::I32))),
            instructions: vec![
                Instruction::I32Const(12),
                Instruction::I32Const(23),
                Instruction::I32Add,
            ],
        }];
        let value = execute_instructions(types, block, vec![]).unwrap();
        assert_eq!(value, vec![Value::I32(35)]);
    }

    #[test]
    fn test_decode_and_exec() {
        let file = std::fs::File::open("test/test.wasm").unwrap();
        let mut reader = std::io::BufReader::new(file);
        let module = decode(&mut reader).unwrap();

        let mut store = Store::default();
        store.instantiate(module);
        let value = store.execute(Idx::new(1), vec![]).unwrap();
        assert_eq!(value, vec![Value::I32(42)]);
    }
}
