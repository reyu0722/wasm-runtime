use super::{
    FuncIdx, FuncType, GlobalIdx, Idx, LabelIdx, LocalIdx, RefType, TableIdx, TypeIdx, ValueType,
};

pub struct Expression {}

pub enum BlockType {
    Type(Idx<FuncType>),
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
    Br(Idx<LabelIdx>),
    BrIf(Idx<LabelIdx>),
    BrTable(Vec<Idx<LabelIdx>>, Idx<LabelIdx>),
    Return,
    Call(Idx<FuncIdx>),
    CallIndirect {
        ty: Idx<TypeIdx>,
        table: Idx<TableIdx>,
    },

    // reference instructions
    RefNull(RefType),
    RefIsNull,
    RefFunc(Idx<FuncIdx>),

    // parametric instructions
    Drop,
    Select(Vec<ValueType>),

    // variable instructions
    LocalGet(Idx<LocalIdx>),
    LocalSet(Idx<LocalIdx>),
    LocalTee(Idx<LocalIdx>),
    GlobalGet(Idx<GlobalIdx>),
    GlobalSet(Idx<GlobalIdx>),

    // table instructions
    Table,

    // memory instructions
    Memory,

    // numeric instructions
    Numeric, // TODO: fix

    // vector instructions
    Vector, // TODO: fix
}
