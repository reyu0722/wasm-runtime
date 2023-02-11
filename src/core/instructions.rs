use super::{RefType, ValueType};

pub struct Expression {}

pub enum BlockType {
    Type(u32),
    ValType(Option<ValueType>),
}

pub enum Instruction {
    // control instructions
    Unreachable,
    Nop,
    Block {
        block_type: BlockType,
        instructions: Vec<Instruction>,
    },
    Loop {
        block_type: BlockType,
        instructions: Vec<Instruction>,
    },
    If {
        block_type: BlockType,
        instructions: Vec<Instruction>,
        else_instructions: Vec<Instruction>,
    },
    Br(u32),
    BrIf(u32),
    BrTable(Vec<u32>, u32),
    Return,
    Call(u32),
    CallIndirect {
        ty: u32,
        table: u32,
    },

    // reference instructions
    RefNull(RefType),
    RefIsNull,
    RefFunc(u32),

    // parametric instructions
    Drop,
    Select,
    SelectT(Vec<ValueType>),

    // variable instructions
    LocalGet(u32),
    LocalSet(u32),
    LocalTee(u32),
    GlobalGet(u32),
    GlobalSet(u32),

    // table instructions
    Table,

    // memory instructions
    Memory,

    // numeric instructions
    Numeric, // TODO: fix

    // vector instructions
    Vector, // TODO: fix
}
