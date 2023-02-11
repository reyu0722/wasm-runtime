use super::ValueType;

pub struct Expression {}

pub enum BlockType {
    Type(u32),
    ValType(Option<ValueType>),
}
