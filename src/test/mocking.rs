use std::any::Any;
use std::sync::Arc;
use std::{f32, u16};
use linked_hash_map::LinkedHashMap;
use rand::distributions::{Alphanumeric, DistString};
use rand::Rng;
use rand::seq::IteratorRandom;
use crate::vm_types::type_info::{ProductType, RecordType, SumType, TypeInfo};
use crate::vm_types::type_sig::TypeSig;
use crate::vm_types::type_tokens;

pub fn mock_object(recursion_depth: u32, is_in_complex_type: bool) -> Result<(Arc<dyn TypeInfo>, Arc<dyn Any>), ()> {
    let random = if recursion_depth <= 3 && !is_in_complex_type {
        rand::thread_rng().gen_range(1..=9)
    } else {
        rand::thread_rng().gen_range(1..=6)
    };
    let mock_product = || -> (ProductType, Vec<Arc<dyn Any>>) {
        let size = rand::thread_rng().gen_range(1..=10);
        let mut vec = Vec::<Arc<dyn Any>>::new();
        let mut type_vec = Vec::new();
        for i in 0..size {
            let (ty, any) = mock_object(recursion_depth + 1, true).unwrap();
            type_vec.push(ty.into());
            vec.push(any);
        }
        (ProductType(type_vec), vec)
    };
    let mock_record = || -> (RecordType, LinkedHashMap<String, Arc<dyn Any>>) {
        let size = rand::thread_rng().gen_range(1..=10);
        let mut data_map = LinkedHashMap::new();
        let mut type_map = LinkedHashMap::new();
        for i in 0..size {
            let (ty, any) = mock_object(recursion_depth + 1, true).unwrap();
            let random_name = Alphanumeric.sample_string(&mut rand::thread_rng(), 10);
            type_map.insert(random_name.clone(), ty.into());
            data_map.insert(random_name.clone(), any);
        }
        (RecordType(Arc::new(type_map)), data_map)
    };
    let mock_sum = || -> (SumType, Vec<Arc<dyn Any>>) {
        let size = rand::thread_rng().gen_range(1..=10);
        let mut type_map = LinkedHashMap::new();
        let mut data_map = LinkedHashMap::new();
        for _ in 0..size {
            let case_name = Alphanumeric.sample_string(&mut rand::thread_rng(), 10);
            let (product, data) = mock_product();
            type_map.insert(case_name.clone(), Arc::new(product));
            data_map.insert(case_name.clone(), data);
        }
        let selected = type_map.keys().choose(&mut rand::thread_rng()).unwrap();
        (SumType(type_map.clone(), selected.clone()), data_map.get(selected).unwrap().clone())
    };

    match random {
        TypeSig::NAT | TypeSig::REFERENCE =>
            Ok((Arc::new(type_tokens::NAT), Arc::new(rand::thread_rng().gen_range(u8::MIN..=u8::MAX) as u64))),
        TypeSig::INT =>
            Ok((Arc::new(type_tokens::INT), Arc::new(rand::thread_rng().gen_range(i8::MIN..=i8::MAX) as i64))),
        TypeSig::DOUBLE =>
            Ok((Arc::new(type_tokens::DOUBLE), Arc::new(rand::thread_rng().gen_range(-1000f64..=1000f64)))),
        TypeSig::CHAR =>
            Ok((Arc::new(type_tokens::CHAR), Arc::new(rand::thread_rng().gen_range('a'..='z')))),
        TypeSig::BOOL =>
            Ok((Arc::new(type_tokens::BOOL), Arc::new(rand::thread_rng().gen_bool(0.5)))),
        TypeSig::PRODUCT => {
            let (ty, list) = mock_product();
            Ok((Arc::new(ty), Arc::new(list)))
        },
        TypeSig::RECORD => {
            let (ty, map) = mock_record();
            Ok((Arc::new(ty), Arc::new(map)))
        },
        TypeSig::SUM => {
            let (ty, list) = mock_sum();
            Ok((Arc::new(ty), Arc::new(list)))
        }
        _ => Err(())
    }
}