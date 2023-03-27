use crate::types::{TxInt, TxFloat};

#[derive(Clone, Copy, PartialEq)]
pub enum Value {
    None,
    Nil,
    Bool(bool),
    Int(TxInt),
    Float(TxFloat),
    Char(char),
    Object(usize),
}
