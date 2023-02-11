use super::ValueType;

pub struct Expression {}

pub enum BlockType {
    Type(u32),
    ValType(Option<ValueType>),
}

pub enum Instruction {
    // control instructions
    Unreachable,
    Nop,
    Block,
    Loop,
    If,
    IfElse,
    Br,
    BrIf,
    BrTable,
    Return,
    Call,
    CallIndirect,

    // reference instructions
    RefNull,
    RefIsNull,
    RefFunc,

    // parametric instructions
    Drop,
    Select,

    // variable instructions
    Variable,

    // table instructions
    Table,

    // memory instructions
    Memory,

    // numeric instructions
    Numeric, // TODO: fix

    // vector instructions
    Vector, // TODO: fix
}
