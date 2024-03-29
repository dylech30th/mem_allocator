use std::any::Any;
use std::mem::size_of;
use std::sync::Arc;
use linked_hash_map::LinkedHashMap;
use crate::allocator::object_allocator::ObjectHeader;
use crate::vm_types::type_info::{SumType, TypeInfo};
use crate::vm_types::type_kind::TypeKind;

pub fn format_heterogeneous_list(list: &Vec<Arc<dyn Any>>) -> String {
    let mut vec: Vec<String> = vec![];
    for item in list {
        if let Some(integer) = item.downcast_ref::<i64>() {
            vec.push(integer.to_string());
        } else if let Some(natural) = item.downcast_ref::<u64>() {
            vec.push(natural.to_string());
        } else if let Some(reference) = item.downcast_ref::<usize>() {
            vec.push(format!("{:x?}", reference));
        } else if let Some(double) = item.downcast_ref::<f64>() {
            vec.push(double.to_string());
        } else if let Some(character) = item.downcast_ref::<char>() {
            vec.push(character.to_string());
        } else if let Some(boolean) = item.downcast_ref::<bool>() {
            vec.push(boolean.to_string());
        } else {
            vec.push("Unknown type".to_string());
        }
    }
    format!("[{}]", vec.join(", "))
}

pub fn format_heterogeneous_map(map: &LinkedHashMap<String, Arc<dyn Any>>) -> String {
    let mut vec: Vec<String> = vec![];
    for (name, item) in map {
        if let Some(integer) = item.downcast_ref::<i64>() {
            vec.push(format!("{}: {}", name, integer));
        } else if let Some(natural) = item.downcast_ref::<u64>() {
            vec.push(format!("{}: {}", name, natural));
        } else if let Some(reference) = item.downcast_ref::<usize>() {
            vec.push(format!("{}: {:x?}", name, reference));
        } else if let Some(double) = item.downcast_ref::<f64>() {
            vec.push(format!("{}: {}", name, double));
        } else if let Some(character) = item.downcast_ref::<char>() {
            vec.push(format!("{}: {}", name, character));
        } else if let Some(boolean) = item.downcast_ref::<bool>() {
            vec.push(format!("{}: {}", name, boolean));
        } else {
            vec.push("Unknown type".to_string());
        }
    }
    format!("[{}]", vec.join(", "))
}

pub unsafe fn format_read_object(tuple: &(Arc<dyn TypeInfo>, Arc<dyn Any>)) -> String {
    let (ty, data) = tuple;
    match ty.kind() {
        TypeKind::Nat =>
            format!("Type: {}, données: {}", ty.name(), data.downcast_ref_unchecked::<u64>()),
        TypeKind::Reference =>
            format!("Type: {}, données: {:x?}", ty.name(), data.downcast_ref_unchecked::<usize>()),
        TypeKind::Int =>
            format!("Type: {}, données: {}", ty.name(), data.downcast_ref_unchecked::<i64>()),
        TypeKind::Double =>
            format!("Type: {}, données: {}", ty.name(), data.downcast_ref_unchecked::<f64>()),
        TypeKind::Char =>
            format!("Type: {}, données: {}", ty.name(), data.downcast_ref_unchecked::<char>()),
        TypeKind::Bool =>
            format!("Type: {}, données: {}", ty.name(), data.downcast_ref_unchecked::<bool>()),
        TypeKind::Product =>
            format!("Type: {}, données: {}", ty.name(), format_heterogeneous_list(data.downcast_ref::<Vec<Arc<dyn Any>>>().unwrap())),
        TypeKind::Record =>
            format!("Type: {}, données: {}", ty.name(), format_heterogeneous_map(data.downcast_ref::<LinkedHashMap<String, Arc<dyn Any>>>().unwrap())),
        TypeKind::Sum =>
            format!("Type: {}, choisi: {}, données: {}",
                    ty.name(),
                    ty.as_any().downcast_ref_unchecked::<SumType>().1,
                    format_heterogeneous_list(data.downcast_ref::<Vec<Arc<dyn Any>>>().unwrap())),
    }
}

pub fn object_size(data_size: usize) -> usize {
    size_of::<ObjectHeader>() + data_size
}

pub fn count_bits_set_range(i: u8, lo_include: usize, hi: usize) -> Vec<usize> {
    let mut set_bits = vec![];
    let mut bit = lo_include;
    while bit < hi {
        if i >> bit & 1 == 1 {
            set_bits.push(bit);
        }
        bit += 1;
    }
    set_bits
}

pub fn count_bits_set(i: u8) -> Vec<usize> {
    let mut set_bits = vec![];
    let mut bit = 0;
    while bit < 8 {
        if i >> bit & 1 == 1 {
            set_bits.push(bit);
        }
        bit += 1;
    }
    set_bits
}

pub fn bit_set(i: u8, bit: usize) -> bool {
    i >> bit & 1 == 1
}