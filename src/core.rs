mod instructions;
mod types;
mod value;
pub use instructions::*;
pub use types::*;
pub use value::*;

#[derive(Default)]
pub struct Module {
    pub types: Vec<FuncType>,
    pub imports: Vec<Import>,
    pub exports: Vec<Export>,
    pub tables: Vec<TableType>,
    pub memories: Vec<MemoryType>,
    pub globals: Vec<Global>,
    pub funcs: Vec<Func>,
    pub start: Option<u32>,
    pub elements: Vec<Element>,
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

pub struct Export {
    pub name: Name,
    pub desc: ExportDesc,
}

pub enum ExportDesc {
    Func(u32),
    Table(u32),
    Memory(u32),
    Global(u32),
}

pub struct Global {
    pub global_type: GlobalType,
    pub init: Expression,
}

pub struct Func {
    pub type_id: u32,
    pub locals: Vec<ValueType>,
    pub body: Expression,
}

pub struct Element {
    pub ty: RefType,
    pub init: Vec<Expression>,
    pub mode: ElementMode,
}

pub enum ElementMode {
    Active { table: u32, offset: Expression },
    Passive,
    Declarative,
}
