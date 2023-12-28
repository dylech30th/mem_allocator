#[derive(Debug)]
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