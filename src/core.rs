mod index;
mod instructions;
mod types;
mod value;

pub use index::*;
pub use instructions::*;
pub use types::*;
pub use value::*;

#[derive(Default)]
pub struct Module {
    pub types: Vec<FuncType>,
    pub imports: Vec<Import>,
    pub exports: Vec<Export>,
    pub tables: Vec<Table>,
    pub memories: Vec<Memory>,
    pub globals: Vec<Global>,
    pub funcs: Vec<Func>,
    pub start: Option<Idx<FuncIdx>>,
    pub elements: Vec<Element>,
}

pub struct Import {
    pub module: Name,
    pub name: Name,
    pub desc: ImportDesc,
}

pub enum ImportDesc {
    Func(Idx<FuncIdx>),
    Table(TableType),
    Memory(Limits),
    Global(GlobalType),
}

pub struct Export {
    pub name: Name,
    pub desc: ExportDesc,
}

pub struct Table(pub TableType);
pub struct Memory(pub MemoryType);

pub enum ExportDesc {
    Func(Idx<FuncIdx>),
    Table(Idx<TableIdx>),
    Memory(Idx<MemIdx>),
    Global(Idx<GlobalIdx>),
}

pub struct Global {
    pub global_type: GlobalType,
    pub init: Expression,
}

pub struct Func {
    pub type_id: Idx<TypeIdx>,
    pub locals: Vec<ValueType>,
    pub body: Expression,
}

pub struct Element {
    pub ty: RefType,
    pub init: Vec<Expression>,
    pub mode: ElementMode,
}

pub enum ElementMode {
    Active {
        table: Idx<TableIdx>,
        offset: Expression,
    },
    Passive,
    Declarative,
}
