use super::Value;
use crate::core::{Idx, Instruction, LocalIdx, ValueType};
use anyhow::{bail, ensure, Result};
use std::collections::VecDeque;

pub enum StackEntry<'a> {
    Value(Value),
    Label(Label<'a>),
    // Frame(Frame),
}

pub struct Label<'a> {
    _arity: usize,
    _instr: &'a Vec<Instruction>,
}

impl<'a> Label<'a> {
    pub fn new(arity: usize, instr: &'a Vec<Instruction>) -> Self {
        Label {
            _arity: arity,
            _instr: instr,
        }
    }
}

#[derive(Default)]
pub struct Stack<'a> {
    data: VecDeque<StackEntry<'a>>,
}

impl<'a> Stack<'a> {
    pub fn push_value(&mut self, value: Value) {
        self.data.push_front(StackEntry::Value(value));
    }

    pub fn push_label(&mut self, label: Label<'a>) {
        self.data.push_front(StackEntry::Label(label));
    }

    pub fn push_i32(&mut self, value: i32) {
        self.push_value(Value::I32(value));
    }

    pub fn push_i64(&mut self, value: i64) {
        self.push_value(Value::I64(value));
    }

    pub fn pop_value(&mut self) -> Result<Value> {
        let Some(StackEntry::Value(value)) = self.data.pop_front() else {
            bail!("expected value on stack");
        };

        Ok(value)
    }

    pub fn pop_and_check_value(&mut self, ty: ValueType) -> Result<Value> {
        let value = self.pop_value()?;
        ensure!(
            value.get_type() == ty,
            "failed to pop value: expected type {:?}",
            ty
        );
        Ok(value)
    }

    pub fn pop_and_check_values(&mut self, types: &[ValueType]) -> Result<Vec<Value>> {
        types
            .iter()
            .map(|ty| self.pop_and_check_value(*ty))
            .collect::<Result<Vec<Value>>>()
    }

    pub fn pop_i32(&mut self) -> Result<i32> {
        let Ok(Value::I32(value)) = self.pop_value() else {
            bail!("expected i32 on stack");
        };

        Ok(value)
    }

    pub fn pop_i64(&mut self) -> Result<i64> {
        let Ok(Value::I64(value)) = self.pop_value() else {
            bail!("expected i64 on stack");
        };

        Ok(value)
    }

    pub fn pop_label(&mut self) -> Result<Label<'a>> {
        let Some(StackEntry::Label(label)) = self.data.pop_front() else {
            bail!("expected label on stack");
        };

        Ok(label)
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

#[derive(Default)]
pub struct Frame {
    locals: Vec<Value>,
}

impl Frame {
    pub fn new(locals: Vec<Value>) -> Self {
        Frame { locals }
    }

    pub fn get_local(&self, idx: Idx<LocalIdx>) -> Value {
        self.locals[idx.get() as usize]
    }

    pub fn set_local(&mut self, idx: Idx<LocalIdx>, value: Value) {
        if idx.get() as usize >= self.locals.len() {
            self.locals.resize(idx.get() as usize + 1, Value::I32(0));
        }

        self.locals[idx.get() as usize] = value;
    }
}
