use super::{
    DataIdx, FuncIdx, FuncType, GlobalIdx, Idx, LabelIdx, LocalIdx, RefType, TableIdx, TypeIdx,
    ValueType,
};

#[derive(Default)]
pub struct Expression {
    pub instructions: Vec<Instruction>,
}

#[derive(Clone, Debug)]
pub enum BlockType {
    Type(Idx<FuncType>),
    ValType(Option<ValueType>),
}

#[derive(Clone, Debug)]
pub struct MemArg {
    pub align: u32,
    pub offset: u32,
}

#[derive(Clone, Debug)]
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
    // TODO: add

    // memory instructions
    I32Load(MemArg),
    I64Load(MemArg),
    F32Load(MemArg),
    F64Load(MemArg),
    I32Load8S(MemArg),
    I32Load8U(MemArg),
    I32Load16S(MemArg),
    I32Load16U(MemArg),
    I64Load8S(MemArg),
    I64Load8U(MemArg),
    I64Load16S(MemArg),
    I64Load16U(MemArg),
    I64Load32S(MemArg),
    I64Load32U(MemArg),
    I32Store(MemArg),
    I64Store(MemArg),
    F32Store(MemArg),
    F64Store(MemArg),
    I32Store8(MemArg),
    I32Store16(MemArg),
    I64Store8(MemArg),
    I64Store16(MemArg),
    I64Store32(MemArg),
    MemorySize,
    MemoryGrow,
    MemoryInit(Idx<DataIdx>),
    DataDrop(Idx<DataIdx>),
    MemoryCopy,
    MemoryFill,

    // numeric instructions
    I32Const(i32),
    I32Add,
    I32Sub,
    I32Mul,
    I32LtS,
    I32DivS,
    Numeric, // TODO: fix

    // vector instructions
    Vector, // TODO: fix
}
