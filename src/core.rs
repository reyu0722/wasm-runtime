use std::marker::PhantomData;

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
    pub tables: Vec<Table>,
    pub memories: Vec<Memory>,
    pub globals: Vec<Global>,
    pub funcs: Vec<Func>,
    pub start: Option<Index<Func>>,
    pub elements: Vec<Element>,
}

pub struct Import {
    pub module: Name,
    pub name: Name,
    pub desc: ImportDesc,
}

pub enum ImportDesc {
    Func(Index<Func>),
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
    Func(Index<Func>),
    Table(Index<Table>),
    Memory(Index<Memory>),
    Global(Index<Global>),
}

pub struct Global {
    pub global_type: GlobalType,
    pub init: Expression,
}

pub struct Func {
    pub type_id: Index<FuncType>,
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
        table: Index<Table>,
        offset: Expression,
    },
    Passive,
    Declarative,
}

pub struct Index<T> {
    pub index: u32,
    _phantom: PhantomData<fn() -> T>,
}

impl<T> Index<T> {
    pub fn new(index: u32) -> Self {
        Self {
            index,
            _phantom: PhantomData,
        }
    }
}

impl<T> From<u32> for Index<T> {
    fn from(index: u32) -> Self {
        Self::new(index)
    }
}
