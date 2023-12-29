use crate::vm_types::type_sig::TypeSig;

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

impl TypeKind {
    pub fn to_type_sig(&self) -> usize {
        match self {
            TypeKind::Nat => TypeSig::NAT,
            TypeKind::Int => TypeSig::INT,
            TypeKind::Reference => TypeSig::REFERENCE,
            TypeKind::Double => TypeSig::DOUBLE,
            TypeKind::Char => TypeSig::CHAR,
            TypeKind::Bool => TypeSig::BOOL,
            TypeKind::Product => TypeSig::PRODUCT,
            TypeKind::Record => TypeSig::RECORD,
            TypeKind::Sum => TypeSig::SUM
        }
    }
}