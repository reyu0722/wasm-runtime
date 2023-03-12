use crate::core::{
    BlockType, Export, ExportDesc, Func, FuncIdx, FuncType, IBinOp, IRelOp, IUnOp, Idx,
    Instruction, Module, NumType, TypeIdx, ValueType,
};
use anyhow::{bail, ensure, Result};
use std::rc::Rc;

mod stack;
use stack::{Frame, Label, Stack};

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

enum ExecuteLabelRes {
    Continue,
    Branch(u32),
}

#[derive(Default)]
pub struct Store {
    funcs: Vec<FuncInstance>,
    exports: Vec<Export>,
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

        for e in module.exports {
            self.exports.push(e);
        }

        instance
    }

    pub fn instantiate(&mut self, module: Module) {
        self.alloc_module(module);
    }

    pub fn invoke(&self, name: &str, args: Vec<Value>) -> Result<Vec<Value>> {
        for e in &self.exports {
            if e.name.as_str() == name {
                if let ExportDesc::Func(idx) = e.desc {
                    return self.execute(idx, args);
                }
            }
        }

        bail!("function not found: {}", name)
    }

    pub fn execute(&self, idx: Idx<FuncIdx>, args: Vec<Value>) -> Result<Vec<Value>> {
        let mut stack = Stack::default();
        let res = self.execute_func(&mut stack, idx, args);
        ensure!(stack.is_empty(), "stack is not empty");
        res
    }

    fn execute_func<'a>(
        &'a self,
        stack: &mut Stack<'a>,
        idx: Idx<FuncIdx>,
        locals: Vec<Value>,
    ) -> Result<Vec<Value>> {
        let func = &self.funcs[idx.get() as usize];

        let mut frame = Frame::new(locals);
        self.execute_label(stack, &mut frame, &func.code.body.instructions)?;

        let values = stack.pop_and_check_values(&func.ty.results)?;
        Ok(values)
    }

    fn get_block_return_type(&self, block_type: &BlockType) -> Vec<ValueType> {
        match block_type {
            BlockType::ValType(op) => op.iter().copied().collect(),
            _ => todo!(),
        }
    }

    fn execute_label<'a>(
        &'a self,
        stack: &mut Stack<'a>,
        frame: &mut Frame,
        instructions: &'a Vec<Instruction>,
    ) -> Result<ExecuteLabelRes> {
        for instr in instructions {
            match instr {
                // control instructions
                Instruction::Nop => {}
                Instruction::Unreachable => bail!("unreachable"),
                Instruction::Block {
                    block_type,
                    instructions,
                } => {
                    let iter = self.get_block_return_type(block_type);
                    let label = Label::new(iter.len(), instructions);
                    stack.push_label(label);

                    let l = self.execute_label(stack, frame, instructions)?;

                    let values = stack.pop_and_check_values(&iter)?;

                    stack.pop_label()?;
                    for v in values {
                        stack.push_value(v);
                    }

                    match l {
                        ExecuteLabelRes::Continue => {}
                        ExecuteLabelRes::Branch(l) => {
                            if l != 0 {
                                return Ok(ExecuteLabelRes::Branch(l - 1));
                            }
                        }
                    }
                }
                Instruction::Loop {
                    block_type,
                    instructions,
                } => {
                    let ty = self.get_block_return_type(block_type);

                    loop {
                        let label = Label::new(ty.len(), instructions);
                        stack.push_label(label);

                        let l = self.execute_label(stack, frame, instructions)?;

                        let values = stack.pop_and_check_values(&ty)?;

                        stack.pop_label()?;
                        for v in values {
                            stack.push_value(v);
                        }

                        match l {
                            ExecuteLabelRes::Continue => {
                                break;
                            }
                            ExecuteLabelRes::Branch(l) => {
                                if l != 0 {
                                    return Ok(ExecuteLabelRes::Branch(l - 1));
                                }
                            }
                        }
                    }
                }
                Instruction::If {
                    block_type,
                    instructions,
                    else_instructions,
                } => {
                    let b = stack.pop_i32()?;
                    let instr = if b != 0 {
                        instructions
                    } else {
                        else_instructions
                    };

                    let iter = self.get_block_return_type(block_type);
                    let label = Label::new(iter.len(), instr);
                    stack.push_label(label);

                    let l = self.execute_label(stack, frame, instr)?;

                    let values = stack.pop_and_check_values(&iter)?;

                    stack.pop_label()?;
                    for v in values {
                        stack.push_value(v);
                    }

                    match l {
                        ExecuteLabelRes::Continue => {}
                        ExecuteLabelRes::Branch(l) => {
                            if l != 0 {
                                return Ok(ExecuteLabelRes::Branch(l - 1));
                            }
                        }
                    }
                }
                Instruction::Br(idx) => return Ok(ExecuteLabelRes::Branch((*idx).into())),
                Instruction::BrIf(idx) => {
                    let b = stack.pop_i32()?;
                    if b != 0 {
                        return Ok(ExecuteLabelRes::Branch((*idx).into()));
                    }
                }
                Instruction::Call(idx) => {
                    let ty = &self.funcs[idx.get() as usize].ty;

                    let args = ty
                        .params
                        .iter()
                        .map(|t| {
                            let v = stack.pop_value()?;
                            ensure!(v.get_type() == *t, "type mismatch");
                            Ok(v)
                        })
                        .collect::<Result<Vec<Value>>>()?;

                    let res = self.execute_func(stack, *idx, args)?;
                    for v in res {
                        stack.push_value(v);
                    }
                }

                Instruction::LocalGet(idx) => {
                    let v = frame.get_local(*idx);
                    stack.push_value(v);
                }
                Instruction::LocalSet(idx) => {
                    let v = stack.pop_value()?;
                    frame.set_local(*idx, v);
                }
                Instruction::LocalTee(idx) => {
                    let v = stack.pop_value()?;
                    frame.set_local(*idx, v);
                    stack.push_value(v);
                }

                Instruction::I32Const(i) => {
                    stack.push_i32(*i);
                }
                Instruction::I32UnOp(op) => {
                    let v = stack.pop_i32()?;
                    let res = match op {
                        IUnOp::Clz => v.leading_zeros(),
                        IUnOp::Ctz => v.trailing_zeros(),
                        IUnOp::Popcnt => v.count_ones(),
                    };
                    stack.push_i32(res as i32);
                }
                Instruction::I32Eqz => {
                    let v = stack.pop_i32()?;
                    stack.push_i32((v == 0).into());
                }
                Instruction::I32BinOp(op) => {
                    let v2 = stack.pop_i32()?;
                    let v1 = stack.pop_i32()?;

                    let res = match op {
                        IBinOp::Add => v1.wrapping_add(v2),
                        IBinOp::Sub => v1.wrapping_sub(v2),
                        IBinOp::Mul => v1.wrapping_mul(v2),
                        IBinOp::DivS => v1.wrapping_div(v2),
                        IBinOp::DivU => (v1 as u32 / v2 as u32) as i32,
                        IBinOp::RemS => v1.wrapping_rem(v2),
                        IBinOp::RemU => (v1 as u32 % v2 as u32) as i32,
                        IBinOp::And => v1 & v2,
                        IBinOp::Or => v1 | v2,
                        IBinOp::Xor => v1 ^ v2,
                        IBinOp::Shl => v1.wrapping_shl(v2 as u32),
                        IBinOp::ShrS => v1.wrapping_shr(v2 as u32),
                        IBinOp::ShrU => (v1 as u32).wrapping_shr(v2 as u32) as i32,
                        IBinOp::Rotl => v1.rotate_left(v2 as u32),
                        IBinOp::Rotr => v1.rotate_right(v2 as u32),
                    };

                    stack.push_i32(res);
                }
                Instruction::I32RelOp(op) => {
                    let v2 = stack.pop_i32()?;
                    let v1 = stack.pop_i32()?;

                    let res = match op {
                        IRelOp::Eq => v1 == v2,
                        IRelOp::Ne => v1 != v2,
                        IRelOp::LtS => v1 < v2,
                        IRelOp::LtU => (v1 as u32) < v2 as u32,
                        IRelOp::GtS => v1 > v2,
                        IRelOp::GtU => v1 as u32 > v2 as u32,
                        IRelOp::LeS => v1 <= v2,
                        IRelOp::LeU => v1 as u32 <= v2 as u32,
                        IRelOp::GeS => v1 >= v2,
                        IRelOp::GeU => v1 as u32 >= v2 as u32,
                    };

                    stack.push_i32(res as i32);
                }

                _ => unimplemented!("{:?}", instr),
            }
        }
        Ok(ExecuteLabelRes::Continue)
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
            Instruction::I32BinOp(IBinOp::Add),
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
            Instruction::I32BinOp(IBinOp::Add),
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
                Instruction::I32BinOp(IBinOp::Add),
            ],
        }];
        let value = execute_instructions(types, block, vec![]).unwrap();
        assert_eq!(value, vec![Value::I32(35)]);
    }

    #[test]
    fn test_loop() {
        let types = vec![FuncType {
            params: vec![ValueType::Num(NumType::I32)],
            results: vec![ValueType::Num(NumType::I32)],
        }];
        let loop_instr = vec![
            Instruction::I32Const(0),
            Instruction::LocalSet(Idx::from(1)),
            Instruction::I32Const(0),
            Instruction::LocalSet(Idx::from(2)),
            Instruction::Loop {
                block_type: BlockType::ValType(None),
                instructions: vec![
                    Instruction::LocalGet(Idx::from(1)),
                    Instruction::I32Const(1),
                    Instruction::I32BinOp(IBinOp::Add),
                    Instruction::LocalSet(Idx::from(1)),
                    Instruction::LocalGet(Idx::from(2)),
                    Instruction::LocalGet(Idx::from(1)),
                    Instruction::I32BinOp(IBinOp::Add),
                    Instruction::LocalSet(Idx::from(2)),
                    Instruction::LocalGet(Idx::from(1)),
                    Instruction::LocalGet(Idx::from(0)),
                    Instruction::I32RelOp(IRelOp::LtS),
                    Instruction::BrIf(Idx::from(0)),
                ],
            },
            Instruction::LocalGet(Idx::from(2)),
        ];
        let value = execute_instructions(types, loop_instr, vec![Value::I32(10)]).unwrap();
        assert_eq!(value, vec![Value::I32(55)]);
    }

    #[test]
    fn test_if() {
        let types = vec![FuncType {
            params: vec![ValueType::Num(NumType::I32)],
            results: vec![ValueType::Num(NumType::I32)],
        }];

        let if_instr = vec![
            Instruction::LocalGet(Idx::from(0)),
            Instruction::If {
                block_type: BlockType::ValType(Some(ValueType::Num(NumType::I32))),
                instructions: vec![Instruction::I32Const(42)],
                else_instructions: vec![Instruction::I32Const(24)],
            },
        ];

        let value =
            execute_instructions(types.clone(), if_instr.clone(), vec![Value::I32(1)]).unwrap();
        assert_eq!(value, vec![Value::I32(42)]);

        let value = execute_instructions(types, if_instr, vec![Value::I32(0)]).unwrap();
        assert_eq!(value, vec![Value::I32(24)]);
    }

    #[test]
    fn test_br() {
        let types = vec![FuncType {
            params: vec![],
            results: vec![ValueType::Num(NumType::I32)],
        }];

        let br = vec![Instruction::Block {
            block_type: BlockType::ValType(Some(ValueType::Num(NumType::I32))),
            instructions: vec![
                Instruction::I32Const(1),
                Instruction::If {
                    block_type: BlockType::ValType(Some(ValueType::Num(NumType::I32))),
                    instructions: vec![Instruction::I32Const(12), Instruction::Br(Idx::from(1))],
                    else_instructions: vec![Instruction::I32Const(12)],
                },
                Instruction::I32Const(42),
                Instruction::I32BinOp(IBinOp::Add),
            ],
        }];

        let value = execute_instructions(types, br, vec![]).unwrap();
        assert_eq!(value, vec![Value::I32(12)]);
    }

    #[test]
    fn test_call() {
        let types = vec![FuncType {
            params: vec![ValueType::Num(NumType::I32)],
            results: vec![ValueType::Num(NumType::I32)],
        }];

        let fib = vec![
            Instruction::LocalGet(Idx::from(0)),
            Instruction::I32Const(3),
            Instruction::I32RelOp(IRelOp::LtS),
            Instruction::If {
                block_type: BlockType::ValType(Some(ValueType::Num(NumType::I32))),
                instructions: vec![Instruction::I32Const(1)],
                else_instructions: vec![
                    Instruction::LocalGet(Idx::from(0)),
                    Instruction::I32Const(-1),
                    Instruction::I32BinOp(IBinOp::Add),
                    Instruction::Call(Idx::from(0)),
                    Instruction::LocalGet(Idx::from(0)),
                    Instruction::I32Const(-2),
                    Instruction::I32BinOp(IBinOp::Add),
                    Instruction::Call(Idx::from(0)),
                    Instruction::I32BinOp(IBinOp::Add),
                ],
            },
        ];

        let value = execute_instructions(types.clone(), fib.clone(), vec![Value::I32(5)]).unwrap();
        assert_eq!(value, vec![Value::I32(5)]);

        let value = execute_instructions(types, fib, vec![Value::I32(10)]).unwrap();
        assert_eq!(value, vec![Value::I32(55)]);
    }

    #[test]
    fn test_decode_and_exec() {
        let file = std::fs::File::open("tests/test.wasm").unwrap();
        let mut reader = std::io::BufReader::new(file);
        let module = decode(&mut reader).unwrap();

        let mut store = Store::default();
        store.instantiate(module);
        let value = store.execute(Idx::new(1), vec![]).unwrap();
        assert_eq!(value, vec![Value::I32(42)]);
    }

    #[test]
    fn test_exec_add() {
        let file = std::fs::File::open("tests/add.wasm").unwrap();
        let mut reader = std::io::BufReader::new(file);
        let module = decode(&mut reader).unwrap();

        let mut store = Store::default();
        store.instantiate(module);
        let value = store
            .execute(Idx::new(1), vec![Value::I32(12), Value::I32(23)])
            .unwrap();
        assert_eq!(value, vec![Value::I32(35)]);
    }

    #[test]
    fn test_exec_combination() {
        let file = std::fs::File::open("tests/combination.wasm").unwrap();
        let mut reader = std::io::BufReader::new(file);
        let module = decode(&mut reader).unwrap();

        let mut store = Store::default();
        store.instantiate(module);
        let value = store
            .execute(Idx::new(1), vec![Value::I32(10), Value::I32(3)])
            .unwrap();
        assert_eq!(value, vec![Value::I32(120)]);
    }
}
