use std::any::Any;
use std::sync::Arc;

pub fn print_heterogeneous_list(list: &Vec<Arc<dyn Any>>) {
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
    println!("[{}]", vec.join(", "));
}