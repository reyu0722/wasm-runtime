use std::{collections::VecDeque, rc::Rc};

use crate::core::{Func, FuncIdx, FuncType, Idx, Instruction, LocalIdx, Module, TypeIdx};

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
    start: Option<Idx<FuncIdx>>,
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

#[derive(Clone, Copy)]
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

pub struct Stack {
    data: VecDeque<StackEntry>,
}

impl Stack {
    fn push_value(&mut self, value: Value) {
        self.data.push_front(StackEntry::Value(value));
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
            start: module.start,
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

        if let Some(idx) = instance.start {
            self.execute(idx)
        }
    }

    fn execute(&mut self, idx: Idx<FuncIdx>) {
        let func = &self.funcs[idx.get() as usize];
        for _instr in &func.code.body.instructions {
            unimplemented!()
        }
    }
}
