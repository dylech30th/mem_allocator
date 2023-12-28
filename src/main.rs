#![allow(incomplete_features)]
#![feature(ptr_sub_ptr)]
#![feature(downcast_unchecked)]
#![feature(unsized_locals, unsized_fn_params)]

pub(crate) mod allocator;
pub(crate) mod utils;
pub(crate) mod vm_types;

use linked_hash_map::LinkedHashMap;
use std::any::Any;
use std::sync::{Arc};
use crate::allocator::object_allocator::ObjectAllocator;
use crate::utils::io::print_heterogeneous_list;
use crate::vm_types::type_info::*;
use crate::vm_types::type_tokens;

// On a en fait une raison plutôt raisonnable pour ne pas implementer le soi-disant "buddy algorithm"
// D'abord, le buddy algorithm a pour effet de reduire la fragmentation de la mémoire, mais dans un heap
// tous les objets qui on va allouer a pour taille de puissance de 2, donc il n'y a pas de fragmentation
// sauf le rembourrage. Ensuite, le buddy algorithm implemente la reduction de la fragmentation en essayant
// de fusionner les blocs "buddies" après la libération d'un bloc, mais dans un heap, on a le collecteur
// d'ordures, spécifiquement on a un collecteur de "mark-compact", la reduction de fragementation sera
// effectuée dans la phase de fragementation du collecteur d'ordures.

fn main() {
    unsafe {
        let mut allocator = ObjectAllocator::new();
        let res = allocator.write_int(123123123).unwrap();
        let (any, info) = allocator.read_obj(res).unwrap();
        let i = any.downcast_unchecked::<i64>();
        let info = info.name();
        println!("{}", i);
        println!("{}", info);
        let res_nat = allocator.write_nat_or_reference(987987987, false).unwrap();
        let (any_nat, info_nat) = allocator.read_obj(res_nat).unwrap();
        let i_nat = any_nat.downcast_unchecked::<u64>();
        let info_nat = info_nat.name();
        println!("{}", i_nat);
        println!("{}", info_nat);
        let res_double = allocator.write_double(123.123).unwrap();
        let (any_double, info_double) = allocator.read_obj(res_double).unwrap();
        let i_double = any_double.downcast_unchecked::<f64>();
        let info_double = info_double.name();
        println!("{}", i_double);
        println!("{}", info_double);
        let res_char = allocator.write_char('a').unwrap();
        let (any_char, info_char) = allocator.read_obj(res_char).unwrap();
        let i_char = any_char.downcast_unchecked::<char>();
        let info_char = info_char.name();
        println!("{}", i_char);
        println!("{}", info_char);
        let res_bool = allocator.write_bool(true).unwrap();
        let (any_bool, info_bool) = allocator.read_obj(res_bool).unwrap();
        let i_bool = any_bool.downcast_unchecked::<bool>();
        let info_bool = info_bool.name();
        println!("{}", i_bool);
        println!("{}", info_bool);

        let product_type = ProductType(vec![Arc::new(type_tokens::INT), Arc::new(type_tokens::CHAR), Arc::new(type_tokens::BOOL), Arc::new(type_tokens::INT), Arc::new(type_tokens::BOOL), Arc::new(type_tokens::DOUBLE)]);
        println!("size: {}", product_type.size());
        let res_product = allocator.write_product(&[Arc::new(123i64), Arc::new('a'), Arc::new(true), Arc::new(456i64), Arc::new(false), Arc::new(123.123f64)], &product_type).unwrap();
        let (any_product, info_product) = allocator.read_obj(res_product).unwrap();
        let i_product = any_product.downcast_unchecked::<Vec<Arc<dyn Any>>>();
        let info_product = info_product.name();
        print_heterogeneous_list(&*i_product);
        println!("{}", info_product);

        let mut map = LinkedHashMap::<String, Arc<dyn TypeInfo>>::new();
        map.insert("int1".to_string(), Arc::new(type_tokens::INT));
        map.insert("char1".to_string(), Arc::new(type_tokens::CHAR));
        map.insert("bool1".to_string(), Arc::new(type_tokens::BOOL));
        map.insert("int2".to_string(), Arc::new(type_tokens::INT));
        map.insert("bool2".to_string(), Arc::new(type_tokens::BOOL));
        map.insert("double1".to_string(), Arc::new(type_tokens::DOUBLE));
        let record_type = RecordType(Arc::new(map));
        println!("size: {}", record_type.size()); // record type is definitely more compact
        let mut data_map = LinkedHashMap::<String, Arc<dyn Any>>::new();
        data_map.insert("int1".to_string(), Arc::new(123i64));
        data_map.insert("char1".to_string(), Arc::new('a'));
        data_map.insert("bool1".to_string(), Arc::new(true));
        data_map.insert("int2".to_string(), Arc::new(456i64));
        data_map.insert("bool2".to_string(), Arc::new(false));
        data_map.insert("double1".to_string(), Arc::new(123.123f64));
        let res_record = allocator.write_record(&data_map, &record_type).unwrap();
        let (any_record, info_record) = allocator.read_obj(res_record).unwrap();
        let i_record = any_record.downcast_unchecked::<Vec<Arc<dyn Any>>>();
        let info_record = info_record.name();
        print_heterogeneous_list(&*i_record);
        println!("{}", info_record);

        let mut sum_type_map = LinkedHashMap::<String, Arc<ProductType>>::new();
        sum_type_map.insert("Some".to_string(), Arc::new(ProductType(vec![Arc::new(type_tokens::INT)])));
        sum_type_map.insert("None".to_string(), Arc::new(ProductType(vec![])));
        let sum_type = SumType(sum_type_map, "Some".to_string());
        let res_sum = allocator.write_sum(&[Arc::new(123i64)], &sum_type).unwrap();
        let (any_sum, info_sum) = allocator.read_obj(res_sum).unwrap();
        let i_sum = any_sum.downcast_unchecked::<Vec<Arc<dyn Any>>>();
        let info_sum = info_sum.name();
        print_heterogeneous_list(&*i_sum);
        println!("{}", info_sum);
    }
}
