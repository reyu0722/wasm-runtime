use anyhow::{bail, Result};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NumType {
    I32,
    I64,
    F32,
    F64,
}

impl NumType {
    pub fn from_byte(b: u8) -> Result<Self> {
        match b {
            0x7f => Ok(NumType::I32),
            0x7e => Ok(NumType::I64),
            0x7d => Ok(NumType::F32),
            0x7c => Ok(NumType::F64),
            _ => bail!("invalid num type: {}", b),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VecType {
    V128,
}

impl VecType {
    pub fn from_byte(b: u8) -> Result<Self> {
        match b {
            0x7b => Ok(VecType::V128),
            _ => bail!("invalid vec type: {}", b),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RefType {
    Funcref,
    Externref,
}

impl RefType {
    pub fn from_byte(b: u8) -> Result<Self> {
        match b {
            0x70 => Ok(RefType::Funcref),
            0x6f => Ok(RefType::Externref),
            _ => bail!("invalid ref type: {}", b),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ValueType {
    Num(NumType),
    Vec(VecType),
    Ref(RefType),
}

impl ValueType {
    pub fn from_byte(b: u8) -> Result<Self> {
        NumType::from_byte(b)
            .map(ValueType::Num)
            .or_else(|_| VecType::from_byte(b).map(ValueType::Vec))
            .or_else(|_| RefType::from_byte(b).map(ValueType::Ref))
    }
}

pub type ResultType = Vec<ValueType>;

pub struct FuncType {
    pub params: ResultType,
    pub results: ResultType,
}

pub struct Limits {
    pub min: u32,
    pub max: Option<u32>,
}

pub type MemoryType = Limits;

pub struct TableType {
    pub limits: Limits,
    pub elem_type: RefType,
}

pub struct GlobalType {
    pub value_type: ValueType,
    pub mutability: bool,
}
