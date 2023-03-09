mod index;
mod instructions;
mod types;
mod value;

pub use index::*;
pub use instructions::*;
pub use types::*;
pub use value::*;

#[derive(Default, Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct Import {
    pub module: Name,
    pub name: Name,
    pub desc: ImportDesc,
}

#[derive(Clone, Debug)]
pub enum ImportDesc {
    Func(Idx<FuncIdx>),
    Table(TableType),
    Memory(Limits),
    Global(GlobalType),
}

#[derive(Clone, Debug)]
pub struct Export {
    pub name: Name,
    pub desc: ExportDesc,
}

#[derive(Clone, Debug)]
pub struct Table(pub TableType);

#[derive(Clone, Debug)]
pub struct Memory(pub MemoryType);

#[derive(Clone, Debug)]
pub enum ExportDesc {
    Func(Idx<FuncIdx>),
    Table(Idx<TableIdx>),
    Memory(Idx<MemIdx>),
    Global(Idx<GlobalIdx>),
}

#[derive(Clone, Debug)]
pub struct Global {
    pub global_type: GlobalType,
    pub init: Expression,
}

#[derive(Clone, Debug)]
pub struct Func {
    pub type_id: Idx<TypeIdx>,
    pub locals: Vec<ValueType>,
    pub body: Expression,
}

#[derive(Clone, Debug)]
pub struct Element {
    pub ty: RefType,
    pub init: Vec<Expression>,
    pub mode: ElementMode,
}

#[derive(Clone, Debug)]
pub enum ElementMode {
    Active {
        table: Idx<TableIdx>,
        offset: Expression,
    },
    Passive,
    Declarative,
}
