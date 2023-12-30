use std::any::Any;
use std::sync::Arc;
use linked_hash_map::LinkedHashMap;
use crate::allocator::object_allocator::ObjectAllocator;
use crate::test::mocking::ObjectMocker;
use crate::utils::io::{format_heterogeneous_list, format_read_object};
use crate::vm_types::type_info::{ProductType, RecordType, SumType, TypeInfo};
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

pub fn test_obj_allocation_stability() {
    unsafe {
        let mut mocker = ObjectMocker::new();
        // test::object_allocator_test::test_obj_alloc_single(&mut allocator);
        let mut vec = vec![];
        let mut reses = vec![];
        for _ in 1..=100 {
            let res = mocker.mock_and_allocate_object().unwrap();
            reses.push(res.0);
            vec.push(res.1);
        }

        for (i, ptr) in vec.iter().enumerate() {
            let res1 = mocker.allocator.read_obj(*ptr).unwrap();
            let res2 = reses.get(i).unwrap();
            println!("{} == {}: {}", format_read_object(&res1), format_read_object(res2), format_read_object(&res1) == format_read_object(res2));
        }

        println!("Tous les équivalences tiennent: {}", vec.iter().zip(reses.iter()).all(|(x, y)| {
            let res1 = mocker.allocator.read_obj(*x).unwrap();
            let res2 = y;
            format_read_object(&res1) == format_read_object(res2)
        }));
    }
}