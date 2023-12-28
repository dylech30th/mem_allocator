use std::any::Any;
use std::sync::Arc;
use linked_hash_map::LinkedHashMap;

pub fn format_heterogeneous_list(list: &Vec<Arc<dyn Any>>) -> String {
    let mut vec: Vec<String> = vec![];
    for item in list {
        if let Some(integer) = item.downcast_ref::<i64>() {
            vec.push(integer.to_string());
        } else if let Some(natural_or_ref) = item.downcast_ref::<u64>() {
            vec.push(natural_or_ref.to_string());
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
        } else if let Some(natural_or_ref) = item.downcast_ref::<u64>() {
            vec.push(format!("{}: {}", name, natural_or_ref));
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