use std::marker::PhantomData;

#[derive(Debug)]
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

impl<T> From<Idx<T>> for u32 {
    fn from(idx: Idx<T>) -> Self {
        idx.index
    }
}

impl<T> Clone for Idx<T> {
    fn clone(&self) -> Self {
        Self::new(self.index)
    }
}

impl<T> Copy for Idx<T> {}

#[derive(Debug)]
pub struct TypeIdx;
#[derive(Debug)]
pub struct FuncIdx;
#[derive(Debug)]
pub struct TableIdx;
#[derive(Debug)]
pub struct MemIdx;
#[derive(Debug)]
pub struct GlobalIdx;
#[derive(Debug)]
pub struct ElemIdx;
#[derive(Debug)]
pub struct DataIdx;
#[derive(Debug)]
pub struct LabelIdx;
#[derive(Debug)]
pub struct LocalIdx;
