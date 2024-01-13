use std::any::Any;
use std::mem::{align_of, size_of};
use std::sync::Arc;
use linked_hash_map::LinkedHashMap;
use crate::allocator::object_allocator;
use crate::utils::iter_ext::IterExt;
use crate::vm_types::type_kind::TypeKind;
use crate::vm_types::type_sig::TypeSig;

pub trait TypeInfo : Send + Sync {
    fn size(&self) -> usize;
    fn name(&self) -> String;
    fn kind(&self) -> TypeKind;
    fn alignment(&self) -> usize;
    fn as_any(&self) -> &dyn Any;
}

pub struct TypeDeclaration(pub String, pub Box<dyn TypeInfo>);
impl TypeInfo for TypeDeclaration {
    fn size(&self) -> usize {
        self.1.size()
    }

    fn name(&self) -> String {
        String::from(&self.1.name())
    }

    fn kind(&self) -> TypeKind {
        self.1.kind()
    }

    fn alignment(&self) -> usize {
        self.1.alignment()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone)]
pub struct SumType(pub LinkedHashMap<String, Arc<ProductType>>, pub String);

impl SumType {
    pub fn alignment_table(&self) -> Vec<usize> {
        self.0.get(&self.1).unwrap().alignment_table()
    }
}

impl TypeInfo for SumType {
    fn size(&self) -> usize {
        self.0.get(&self.1).unwrap().size()
    }

    fn name(&self) -> String {
        let mut vec = Vec::<String>::new();
        for (name, field) in &self.0 {
            vec.push(format!("{}{}", name, field.name()));
        }
        format!("{{{}}}", vec.join(", "))
    }

    fn kind(&self) -> TypeKind {
        TypeKind::Sum
    }

    fn alignment(&self) -> usize {
        self.0.get(&self.1).unwrap().alignment()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// impl optimized alignment: the sum type's fields are unordered!
#[derive(Clone)]
pub struct RecordType(pub Arc<LinkedHashMap<String, Arc<dyn TypeInfo>>>);
impl RecordType {
    pub(crate) fn alignment_table(&self) -> LinkedHashMap<String, usize> {
        let mut offset = 0;
        let mut alignment_table =  LinkedHashMap::<String, usize>::new();
        let grouped_by_alignment = (*(self.0)).iter().group_by_sorted(|(_, a)| a.alignment());
        let iter = grouped_by_alignment.iter().rev();
        for (_, items) in iter {
            for (name, info) in items {
                alignment_table.insert((*name).clone(), offset);
                offset += info.alignment();
            }
        }
        alignment_table
    }
}

impl TypeInfo for RecordType {
    fn size(&self) -> usize {
        let table = self.alignment_table();
        (table.values().last().unwrap()) + self.0.get(table.keys().last().unwrap()).unwrap().size()
    }

    fn name(&self) -> String {
        let mut vec = Vec::<String>::new();
        for (name, field) in &*self.0 {
            vec.push(format!("{}: {}", name, field.name()));
        }
        format!("{{{}}}", vec.join(", "))
    }

    fn kind(&self) -> TypeKind {
        TypeKind::Record
    }

    fn alignment(&self) -> usize {
        self.0.iter().map(|(_, info)| info.alignment()).max().unwrap()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone)]
pub struct ProductType(pub Vec<Arc<dyn TypeInfo>>);
impl ProductType {

    // this rearranges fields to make it more compact
    pub fn alignment_table(&self) -> Vec<usize> {
        if self.0.is_empty() {
            return vec![];
        }
        let mut offset = 0;
        let mut alignment_table: Vec<usize> =  vec![offset];
        offset += self.0.first().unwrap().size();
        for field in self.0.iter().skip(1) {
            let padding_discriminant = offset % field.alignment();
            let padding = if padding_discriminant == 0 { 0 } else { field.alignment() - padding_discriminant };
            offset += padding;
            alignment_table.push(offset);
            offset += field.size();
        }
        alignment_table
    }
}

impl TypeInfo for ProductType {
    fn size(&self) -> usize {
        (*self.alignment_table().last().unwrap_or(&0)) + self.0.last().map(|x| x.alignment()).unwrap_or(0) // the last field's start + the last field's size
    }

    fn name(&self) -> String {
        format!("({})", self.0.iter().map(|info| info.name()).collect::<Vec<_>>().join(", "))
    }

    fn kind(&self) -> TypeKind {
        TypeKind::Product
    }

    fn alignment(&self) -> usize {
        self.0.iter().map(|info| info.alignment()).max().unwrap_or(0)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Copy, Clone)]
pub struct NatType;
impl TypeInfo for NatType {
    fn size(&self) -> usize {
        size_of::<u64>()
    }

    fn name(&self) -> String {
        String::from("Nat")
    }

    fn kind(&self) -> TypeKind {
        TypeKind::Nat
    }

    fn alignment(&self) -> usize {
        size_of::<u64>()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Copy, Clone)]
pub struct IntType;
impl TypeInfo for IntType {
    fn size(&self) -> usize {
        size_of::<i64>()
    }

    fn name(&self) -> String {
        String::from("Int")
    }

    fn kind(&self) -> TypeKind {
        TypeKind::Int
    }

    fn alignment(&self) -> usize {
        size_of::<i64>()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone, Copy)]
pub struct ReferenceType(pub usize);
impl TypeInfo for ReferenceType {
    fn size(&self) -> usize {
        size_of::<usize>()
    }

    fn name(&self) -> String {
        format!("&{}", TypeSig::type_sig_to_string(self.0)).to_string()
    }

    fn kind(&self) -> TypeKind {
        TypeKind::Reference
    }

    fn alignment(&self) -> usize {
        align_of::<usize>()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Copy, Clone)]
pub struct DoubleType;
impl TypeInfo for DoubleType {
    fn size(&self) -> usize {
        size_of::<f64>()
    }

    fn name(&self) -> String {
        String::from("Double")
    }

    fn kind(&self) -> TypeKind {
        TypeKind::Double
    }

    fn alignment(&self) -> usize {
        size_of::<f64>()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Copy, Clone)]
pub struct CharType;
impl TypeInfo for CharType {
    fn size(&self) -> usize {
        if object_allocator::USE_COMPACT_LAYOUT {
            1
        } else {
            8
        }
    }

    fn name(&self) -> String {
        String::from("Char")
    }

    fn kind(&self) -> TypeKind {
        TypeKind::Char
    }

    fn alignment(&self) -> usize {
        if object_allocator::USE_COMPACT_LAYOUT {
            1
        } else {
            size_of::<usize>()
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Copy, Clone)]
pub struct BoolType;
impl TypeInfo for BoolType {
    fn size(&self) -> usize {
        if object_allocator::USE_COMPACT_LAYOUT {
            1
        } else {
            8
        }
    }

    fn name(&self) -> String {
        String::from("Bool")
    }

    fn kind(&self) -> TypeKind {
        TypeKind::Bool
    }

    fn alignment(&self) -> usize {
        if object_allocator::USE_COMPACT_LAYOUT {
            1
        } else {
            size_of::<usize>()
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}