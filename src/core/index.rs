use std::marker::PhantomData;

pub struct Idx<T> {
    pub index: u32,
    _phantom: PhantomData<fn() -> T>,
}

impl<T> Idx<T> {
    pub fn new(index: u32) -> Self {
        Self {
            index,
            _phantom: PhantomData,
        }
    }

    pub fn get(&self) -> u32 {
        self.index
    }
}

impl<T> From<u32> for Idx<T> {
    fn from(index: u32) -> Self {
        Self::new(index)
    }
}

impl<T> Clone for Idx<T> {
    fn clone(&self) -> Self {
        Self::new(self.index)
    }
}

impl<T> Copy for Idx<T> {}

pub struct TypeIdx;
pub struct FuncIdx;
pub struct TableIdx;
pub struct MemIdx;
pub struct GlobalIdx;
pub struct ElemIdx;
pub struct DataIdx;
pub struct LabelIdx;
pub struct LocalIdx;
