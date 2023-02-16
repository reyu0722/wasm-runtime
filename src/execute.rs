use std::rc::Rc;

use crate::core::{Func, FuncIdx, FuncType, Idx, Module, TypeIdx};

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

pub enum Value {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

#[derive(Default)]
pub struct Stack {
    frames: Vec<Frame>,
}

impl Stack {
    pub fn push_frame(&mut self, frame: Frame) {
        self.frames.push(frame);
    }
}

#[derive(Default)]
pub struct Frame {
    locals: Vec<Value>,
}

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
            start: module.start,
        };

        for func in module.funcs {
            let addr = self.alloc_func(func, &instance);
            instance.func_addrs.push(addr);
        }

        instance
    }

    pub fn instantiate(&mut self, module: Module) {
        let mut stack = Stack::default();
        let instance = self.alloc_module(module);
        let frame = Frame::default();
        stack.push_frame(frame);

        if let Some(idx) = instance.start {
            self.execute(idx)
        }
    }

    fn execute(&mut self, _idx: Idx<FuncIdx>) {}
}
