mod instructions;
mod types;
mod value;
pub use instructions::*;
pub use types::*;
pub use value::*;

pub struct Module {
    pub types: Vec<FuncType>,
    pub imports: Vec<Import>,
    pub tables: Vec<TableType>,
    pub memories: Vec<MemoryType>,
    pub globals: Vec<Global>,
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

pub struct Global {
    pub global_type: GlobalType,
    pub init: Expression,
}
