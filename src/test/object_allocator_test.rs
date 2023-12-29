use std::any::Any;
use std::sync::Arc;
use linked_hash_map::LinkedHashMap;
use rand::Rng;
use crate::allocator::object_allocator::ObjectAllocator;
use crate::utils::io::{format_heterogeneous_list, format_heterogeneous_map};
use crate::vm_types::type_info::{ProductType, RecordType, ReferenceType, SumType, TypeInfo};
use crate::vm_types::type_kind::TypeKind;
use crate::vm_types::type_tokens;

pub unsafe fn test_obj_alloc_single(allocator: &mut ObjectAllocator) {
    let res = allocator.write_int(123123123).unwrap();
    let (info, any) = allocator.read_obj(res).unwrap();
    let i = any.downcast_ref_unchecked::<i64>();
    let info = info.name();
    println!("{}", i);
    println!("{}", info);
    let res_nat = allocator.write_nat(987987987).unwrap();
    let (info_nat, any_nat) = allocator.read_obj(res_nat).unwrap();
    let i_nat = any_nat.downcast_ref_unchecked::<u64>();
    let info_nat = info_nat.name();
    println!("{}", i_nat);
    println!("{}", info_nat);
    let res_double = allocator.write_double(123.123).unwrap();
    let (info_double, any_double) = allocator.read_obj(res_double).unwrap();
    let i_double = any_double.downcast_ref_unchecked::<f64>();
    let info_double = info_double.name();
    println!("{}", i_double);
    println!("{}", info_double);
    let res_char = allocator.write_char('a').unwrap();
    let (info_char, any_char) = allocator.read_obj(res_char).unwrap();
    let i_char = any_char.downcast_ref_unchecked::<char>();
    let info_char = info_char.name();
    println!("{}", i_char);
    println!("{}", info_char);
    let res_bool = allocator.write_bool(true).unwrap();
    let (info_bool, any_bool) = allocator.read_obj(res_bool).unwrap();
    let i_bool = any_bool.downcast_ref_unchecked::<bool>();
    let info_bool = info_bool.name();
    println!("{}", i_bool);
    println!("{}", info_bool);

    let product_type = ProductType(vec![Arc::new(type_tokens::INT), Arc::new(type_tokens::CHAR), Arc::new(type_tokens::BOOL), Arc::new(type_tokens::INT), Arc::new(type_tokens::BOOL), Arc::new(type_tokens::DOUBLE)]);
    println!("size: {}", product_type.size());
    let res_product = allocator.write_product(&[Arc::new(123i64), Arc::new('a'), Arc::new(true), Arc::new(456i64), Arc::new(false), Arc::new(123.123f64)], &product_type).unwrap();
    let (info_product, any_product) = allocator.read_obj(res_product).unwrap();
    let i_product = any_product.downcast_ref_unchecked::<Vec<Arc<dyn Any>>>();
    let info_product = info_product.name();
    println!("{}", format_heterogeneous_list(&*i_product));
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
    let (info_record, any_record) = allocator.read_obj(res_record).unwrap();
    let i_record = any_record.downcast_ref_unchecked::<Vec<Arc<dyn Any>>>();
    let info_record = info_record.name();
    println!("{}", format_heterogeneous_list(&*i_record));
    println!("{}", info_record);

    let mut sum_type_map = LinkedHashMap::<String, Arc<ProductType>>::new();
    sum_type_map.insert("Some".to_string(), Arc::new(ProductType(vec![Arc::new(type_tokens::INT)])));
    sum_type_map.insert("None".to_string(), Arc::new(ProductType(vec![])));
    let sum_type = SumType(sum_type_map, "Some".to_string());
    let res_sum = allocator.write_sum(&[Arc::new(123i64)], &sum_type).unwrap();
    let (info_sum, any_sum) = allocator.read_obj(res_sum).unwrap();
    let i_sum = any_sum.downcast_ref_unchecked::<Vec<Arc<dyn Any>>>();
    let info_sum = info_sum.name();
    println!("{}", format_heterogeneous_list(&*i_sum));
    println!("{}", info_sum);
}

pub unsafe fn test_obj_alloc_batch(allocator: &mut ObjectAllocator, tuple: &(Arc<dyn TypeInfo>, Arc<dyn Any>), allocated_pointers: &mut Vec<*mut usize>) {
    let (ty, data) = tuple;
    let res = match ty.kind() {
        TypeKind::Nat => allocator.write_nat(*data.downcast_ref_unchecked::<u64>()),
        TypeKind::Reference => allocator.write_reference(*data.downcast_ref_unchecked::<usize>(), ty.as_any().downcast_ref_unchecked::<ReferenceType>()),
        TypeKind::Int => allocator.write_int(*data.downcast_ref_unchecked::<i64>()),
        TypeKind::Double => allocator.write_double(*data.downcast_ref::<f64>().unwrap()),
        TypeKind::Char => allocator.write_char(*data.downcast_ref::<char>().unwrap()),
        TypeKind::Bool => allocator.write_bool(*data.downcast_ref::<bool>().unwrap()),
        TypeKind::Product => {
            let prod = ty.as_any().downcast_ref::<ProductType>().unwrap();
            allocator.write_product(data.downcast_ref::<Vec<Arc<dyn Any>>>().unwrap(), prod)
        },
        TypeKind::Record => {
            let record = ty.as_any().downcast_ref::<RecordType>().unwrap();
            allocator.write_record(data.downcast_ref::<LinkedHashMap<String, Arc<dyn Any>>>().unwrap(), record)
        },
        TypeKind::Sum => {
            let sum = ty.as_any().downcast_ref::<SumType>().unwrap();
            allocator.write_sum(data.downcast_ref::<Vec<Arc<dyn Any>>>().unwrap(), sum)
        }
    }.expect("Allocation failed");
    allocated_pointers.push(res);
}

pub unsafe fn format_read_object(tuple: &(Arc<dyn TypeInfo>, Arc<dyn Any>)) -> String {
    let (ty, data) = tuple;
    match ty.kind() {
        TypeKind::Nat =>
            format!("Type: {}, data: {}", ty.name(), data.downcast_ref_unchecked::<u64>()),
        TypeKind::Reference =>
            format!("Type: {}, data: {}", ty.name(), data.downcast_ref_unchecked::<usize>()),
        TypeKind::Int =>
            format!("Type: {}, data: {}", ty.name(), data.downcast_ref_unchecked::<i64>()),
        TypeKind::Double =>
            format!("Type: {}, data: {}", ty.name(), data.downcast_ref_unchecked::<f64>()),
        TypeKind::Char =>
            format!("Type: {}, data: {}", ty.name(), data.downcast_ref_unchecked::<char>()),
        TypeKind::Bool =>
            format!("Type: {}, data: {}", ty.name(), data.downcast_ref_unchecked::<bool>()),
        TypeKind::Product =>
            format!("Type: {}, data: {}", ty.name(), format_heterogeneous_list(data.downcast_ref::<Vec<Arc<dyn Any>>>().unwrap())),
        TypeKind::Record =>
            format!("Type: {}, data: {}", ty.name(), format_heterogeneous_map(data.downcast_ref::<LinkedHashMap<String, Arc<dyn Any>>>().unwrap())),
        TypeKind::Sum =>
            format!("Type: {}, selected: {}, data: {}",
                    ty.name(),
                    ty.as_any().downcast_ref_unchecked::<SumType>().1,
                    format_heterogeneous_list(data.downcast_ref::<Vec<Arc<dyn Any>>>().unwrap())),
    }
}