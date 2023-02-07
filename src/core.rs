mod types;
mod value;
pub use types::*;
pub use value::*;

pub struct Module {
    pub types: Vec<FuncType>,
    pub imports: Vec<Import>,
}

pub struct Import {
    pub module: Name,
    pub name: Name,
    pub desc: ImportDesc,
}

pub enum ImportDesc {
    Func(u32),
    Table(TableType),
    Memory(Limits),
    Global(GlobalType),
}
