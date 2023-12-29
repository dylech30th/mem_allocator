#[derive(Debug, PartialEq)]
pub enum TypeKind {
    Nat,
    Int,
    Reference,
    Double,
    Char,
    Bool,
    Product,
    Record,
    Sum
}