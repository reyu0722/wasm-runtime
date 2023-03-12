use super::{
    DataIdx, FuncIdx, FuncType, GlobalIdx, Idx, LabelIdx, LocalIdx, RefType, TableIdx, TypeIdx,
    ValueType,
};

#[derive(Default, Clone, Debug)]
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

#[derive(Clone)]
pub struct OpCode {
    code: u8,
}

impl std::fmt::Debug for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#x}", self.code)
    }
}

impl From<u8> for OpCode {
    fn from(code: u8) -> Self {
        OpCode { code }
    }
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
    I32Extend8S,
    I32Extend16S,
    I32UnOp(IUnOp),
    I32Eqz,
    I32BinOp(IBinOp),
    I32RelOp(IRelOp),

    I64Const(i64),
    I64UnOp(IUnOp),
    I64BinOp(IBinOp),
    Numeric(OpCode), // TODO: fix

    // vector instructions
    Vector, // TODO: fix
}

#[derive(Clone, Debug)]
pub enum IUnOp {
    Clz,
    Ctz,
    Popcnt,
}

#[derive(Clone, Debug)]
pub enum IBinOp {
    Add,
    Sub,
    Mul,
    DivS,
    DivU,
    RemS,
    RemU,
    And,
    Or,
    Xor,
    Shl,
    ShrS,
    ShrU,
    Rotl,
    Rotr,
}

#[derive(Clone, Debug)]
pub enum IRelOp {
    Eq,
    Ne,
    LtS,
    LtU,
    GtS,
    GtU,
    LeS,
    LeU,
    GeS,
    GeU,
}
