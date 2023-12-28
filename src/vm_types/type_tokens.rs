use crate::vm_types::type_info::*;

#[non_exhaustive]
pub struct TypeTokens;

pub static INT: IntType = IntType{};
pub static NAT: NatType = NatType{};
pub static DOUBLE: DoubleType = DoubleType{};
pub static CHAR: CharType = CharType{};
pub static BOOL: BoolType = BoolType{};