use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use linked_hash_map::LinkedHashMap;
use rand::distributions::{Alphanumeric, DistString};
use rand::Rng;
use rand::seq::IteratorRandom;
use crate::allocator::object_allocator::{ObjectHeader};
use crate::gc::gc::GarbageCollector;
use crate::vm_types::type_info::*;
use crate::vm_types::type_kind::TypeKind;
use crate::vm_types::type_sig::TypeSig;
use crate::vm_types::type_tokens;

// Cette structure nous aide à mocker les reférences
pub struct ObjectMocker {
    pub allocator: Rc<RefCell<GarbageCollector>>,
    pub mocked_objects_ptrs: Vec<(TypeKind, *mut ObjectHeader)>
}

pub struct MockResult(pub (Arc<dyn TypeInfo>, Arc<dyn Any>), pub *mut ObjectHeader);

impl ObjectMocker {
    pub unsafe fn new() -> ObjectMocker {
        ObjectMocker {
            allocator: GarbageCollector::new(),
            mocked_objects_ptrs: Vec::new()
        }
    }

    #[allow(clippy::clone_on_copy)]
    pub unsafe fn mock_and_allocate_object(&mut self) -> Result<MockResult, String> {
        let obj = self.mock_object(0, false)
            .map_err(|_| "Failed to mock object".to_string())?;
        let allocated = self.allocator.borrow_mut().heap.allocate_general(&obj)
            .map_err(|x| format!("Failed to allocate object while mocking: {:?}", x))?;
        // without clone here will cause strange problems
        self.mocked_objects_ptrs.push((obj.0.kind().clone(), allocated));
        Ok(MockResult(obj, allocated))
    }

    fn gen_random(&self, recursion_depth: u32, is_in_complex_type: bool) -> usize {
        loop {
            let random = if recursion_depth <= 3 && !is_in_complex_type {
                rand::thread_rng().gen_range(7..=7)
            } else {
                rand::thread_rng().gen_range(1..=6)
            };
            if random == TypeSig::REFERENCE && self.mocked_objects_ptrs.len() < 10 {
                continue
            } else {
                return random
            }
        }
    }

    #[allow(clippy::type_complexity)]
    unsafe fn mock_reference(&self) -> Result<(Arc<dyn TypeInfo>, Arc<dyn Any>), ()> {
        let (type_kind, ptr) = self.mocked_objects_ptrs.iter().choose(&mut rand::thread_rng()).unwrap();
        let ty = ReferenceType(type_kind.to_type_sig());
        Ok((Arc::new(ty), Arc::new(*ptr as usize)))
    }

    pub unsafe fn mock_object(&self, recursion_depth: u32, is_in_complex_type: bool) -> Result<(Arc<dyn TypeInfo>, Arc<dyn Any>), ()> {
        let random = self.gen_random(recursion_depth, is_in_complex_type);
        let mock_product = || -> (ProductType, Vec<Arc<dyn Any>>) {
            let size = rand::thread_rng().gen_range(1..=10);
            let mut vec = Vec::<Arc<dyn Any>>::new();
            let mut type_vec = Vec::new();
            for _ in 0..size {
                let (ty, any) = self.mock_object(recursion_depth + 1, true).unwrap();
                type_vec.push(ty);
                vec.push(any);
            }
            (ProductType(type_vec), vec)
        };
        let mock_record = || -> (RecordType, LinkedHashMap<String, Arc<dyn Any>>) {
            let size = rand::thread_rng().gen_range(1..=10);
            let mut data_map = LinkedHashMap::new();
            let mut type_map = LinkedHashMap::new();
            for _ in 0..size {
                let (ty, any) = self.mock_object(recursion_depth + 1, true).unwrap();
                let random_name = Alphanumeric.sample_string(&mut rand::thread_rng(), 10);
                type_map.insert(random_name.clone(), ty);
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
            TypeSig::NAT =>
                Ok((Arc::new(type_tokens::NAT), Arc::new(rand::thread_rng().gen_range(u64::MIN..=u64::MAX)))),
            TypeSig::INT =>
                Ok((Arc::new(type_tokens::INT), Arc::new(rand::thread_rng().gen_range(i64::MIN..=i64::MAX)))),
            TypeSig::DOUBLE =>
                Ok((Arc::new(type_tokens::DOUBLE), Arc::new(rand::thread_rng().gen_range(-999999999.9999f64..=999999999.9999f64)))),
            TypeSig::CHAR =>
                Ok((Arc::new(type_tokens::CHAR), Arc::new(rand::thread_rng().gen_range('a'..='z')))),
            TypeSig::BOOL =>
                Ok((Arc::new(type_tokens::BOOL), Arc::new(rand::thread_rng().gen_bool(0.5)))),
            TypeSig::REFERENCE =>
                self.mock_reference(),
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
}