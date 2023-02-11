use super::{Func, FuncType, Global, Index, Label, Local, RefType, Table, ValueType};

pub struct Expression {}

pub enum BlockType {
    Type(Index<FuncType>),
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
    Br(Index<Label>),
    BrIf(Index<Label>),
    BrTable(Vec<Index<Label>>, Index<Label>),
    Return,
    Call(Index<Func>),
    CallIndirect {
        ty: Index<FuncType>,
        table: Index<Table>,
    },

    // reference instructions
    RefNull(RefType),
    RefIsNull,
    RefFunc(Index<Func>),

    // parametric instructions
    Drop,
    Select,
    SelectT(Vec<ValueType>),

    // variable instructions
    LocalGet(Index<Local>),
    LocalSet(Index<Local>),
    LocalTee(Index<Local>),
    GlobalGet(Index<Global>),
    GlobalSet(Index<Global>),

    // table instructions
    Table,

    // memory instructions
    Memory,

    // numeric instructions
    Numeric, // TODO: fix

    // vector instructions
    Vector, // TODO: fix
}
