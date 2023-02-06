pub enum NumType {
    I32,
    I64,
    F32,
    F64,
}

pub enum VecType {
    V128,
}

pub enum RefType {
    Funcref,
    Externref,
}

pub enum ValueType {
    Num(NumType),
    Vec(VecType),
    Ref(RefType),
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
