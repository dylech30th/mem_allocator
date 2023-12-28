use std::{alloc, any};
use std::alloc::Layout;
use std::any::Any;
use std::mem::size_of;
use std::sync::Arc;
use linked_hash_map::LinkedHashMap;
use crate::allocator::mem_allocator::MemAllocator;
use crate::utils::errors::AllocatorError;
use crate::vm_types::type_info::*;
use crate::vm_types::type_kind::TypeKind;
use crate::vm_types::type_sig::TypeSig;
use crate::vm_types::type_tokens;

pub struct ObjectAllocator {
    pub allocator: MemAllocator,
    pub allocated_objects: Vec<*mut usize>
}

impl ObjectAllocator {
    pub fn new() -> Self {
        ObjectAllocator {
            allocator: MemAllocator::new(),
            allocated_objects: Vec::new()
        }
    }

    pub unsafe fn write_int(&mut self, value: i64) -> Result<*mut usize, AllocatorError> {
        let size_required = size_of::<usize>() + size_of::<usize>() + size_of::<i64>();
        let p = self.allocator.alloc(size_required, size_of::<usize>())? as *mut usize;
        p.write(TypeSig::INT);
        p.add(1).write(&type_tokens::INT as *const IntType as usize);
        (p.add(2) as *mut i64).write(value);
        self.allocated_objects.push(p);
        Ok(p)
    }

    pub unsafe fn write_nat_or_reference(&mut self, value: u64, is_ref: bool) -> Result<*mut usize, AllocatorError> {
        let size_required = size_of::<usize>() + size_of::<usize>() + size_of::<u64>();
        let p = self.allocator.alloc(size_required, size_of::<usize>())? as *mut usize;
        if is_ref {
            p.write(TypeSig::REFERENCE);
        } else {
            p.write(TypeSig::NAT);
        }
        p.add(1).write(&type_tokens::NAT as *const NatType as usize);
        (p.add(2) as *mut u64).write(value);
        self.allocated_objects.push(p);
        Ok(p)
    }

    pub unsafe fn write_double(&mut self, value: f64) -> Result<*mut usize, AllocatorError> {
        let size_required = size_of::<usize>() + size_of::<usize>() + size_of::<f64>();
        let p = self.allocator.alloc(size_required, size_of::<usize>())? as *mut usize;
        p.write(TypeSig::DOUBLE);
        p.add(1).write(&type_tokens::DOUBLE as *const DoubleType as usize);
        (p.add(2) as *mut f64).write(value);
        self.allocated_objects.push(p);
        Ok(p)
    }

    pub unsafe fn write_char(&mut self, value: char) -> Result<*mut usize, AllocatorError> {
        let size_required = size_of::<usize>() + size_of::<usize>() + size_of::<char>();
        let p = self.allocator.alloc(size_required, size_of::<usize>())? as *mut usize;
        p.write(TypeSig::CHAR);
        p.add(1).write(&type_tokens::CHAR as *const CharType as usize);
        (p.add(2) as *mut char).write(value);
        self.allocated_objects.push(p);
        Ok(p)
    }

    pub unsafe fn write_bool(&mut self, value: bool) -> Result<*mut usize, AllocatorError> {
        let size_required = size_of::<usize>() + size_of::<usize>() + size_of::<bool>();
        let p = self.allocator.alloc(size_required, size_of::<usize>())? as *mut usize;
        p.write(TypeSig::BOOL);
        p.add(1).write(&type_tokens::BOOL as *const BoolType as usize);
        (p.add(2) as *mut bool).write(value);
        self.allocated_objects.push(p);
        Ok(p)
    }

    // noinspection ALL
    pub unsafe fn write_record(&mut self, data: &LinkedHashMap<String, Arc<dyn Any>>, type_info: &RecordType) -> Result<*mut usize, AllocatorError> {
        let size_required = size_of::<usize>() + size_of::<usize>() + type_info.size();
        let p = self.allocator.alloc(size_required, size_of::<usize>()).unwrap() as *mut usize;
        p.write(TypeSig::RECORD);
        p.add(1).write(self.heap_allocated_type_info(type_info) as usize);
        let data_ptr = p.add(2);
        for (name, field) in data.iter() {
            let field_ptr = data_ptr.add(type_info.alignment_table()[name]);
            if let Some(integer) = field.downcast_ref::<i64>() {
                (field_ptr as *mut i64).write(*integer);
            } else if let Some(natural_or_ref) = field.downcast_ref::<u64>() {
                (field_ptr as *mut u64).write(*natural_or_ref);
            } else if let Some(double) = field.downcast_ref::<f64>() {
                (field_ptr as *mut f64).write(*double);
            } else if let Some(character) = field.downcast_ref::<char>() {
                (field_ptr as *mut char).write(*character);
            } else if let Some(boolean) = field.downcast_ref::<bool>() {
                (field_ptr as *mut bool).write(*boolean);
            } else {
                return Err(AllocatorError::ObjectAllocationFailed("Only primitive types are supported in Product Type".to_string()));
            }
        }
        Ok(p)
    }

    // noinspection ALL
    pub unsafe fn write_product(&mut self, data: &[Arc<dyn Any>], type_info: &ProductType) -> Result<*mut usize, AllocatorError> {
        let size_required = size_of::<usize>() + size_of::<usize>() + type_info.size();
        let p = self.allocator.alloc(size_required, size_of::<usize>()).unwrap() as *mut usize;
        p.write(TypeSig::PRODUCT);
        p.add(1).write(self.heap_allocated_type_info(type_info) as usize);
        self.write_product_data(data, &type_info.alignment_table(), &mut p.add(2))?;
        Ok(p)
    }

    pub unsafe fn write_sum(&mut self, data: &[Arc<dyn Any>], type_info: &SumType) -> Result<*mut usize, AllocatorError> {
        let size_required = size_of::<usize>() + size_of::<usize>() + type_info.size();
        let p = self.allocator.alloc(size_required, size_of::<usize>()).unwrap() as *mut usize;
        p.write(TypeSig::SUM);
        p.add(1).write(self.heap_allocated_type_info(type_info) as usize);
        self.write_product_data(data, &type_info.alignment_table(), &mut p.add(2))?;
        Ok(p)
    }

    // noinspection all
    unsafe fn write_product_data(&self, data: &[Arc<dyn Any>], alignments: &[usize], data_ptr: &mut *mut usize) -> Result<(), AllocatorError> {
        if data.len() != alignments.len() {
            return Err(AllocatorError::ProductSizeMismatch);
        }
        if data.is_empty() {
            return Ok(());
        }
        for (i, field) in data.iter().enumerate() {
            let field_ptr = data_ptr.add(alignments[i]);
            if let Some(integer) = field.downcast_ref::<i64>() {
                (field_ptr as *mut i64).write(*integer);
            } else if let Some(natural_or_ref) = field.downcast_ref::<u64>() {
                (field_ptr as *mut u64).write(*natural_or_ref);
            } else if let Some(double) = field.downcast_ref::<f64>() {
                (field_ptr as *mut f64).write(*double);
            } else if let Some(character) = field.downcast_ref::<char>() {
                (field_ptr as *mut char).write(*character);
            } else if let Some(boolean) = field.downcast_ref::<bool>() {
                (field_ptr as *mut bool).write(*boolean);
            } else {
                return Err(AllocatorError::ObjectAllocationFailed("Only primitive types are supported in Product Type".to_string()));
            }
        }
        Ok(())
    }

    unsafe fn heap_allocated_type_info<T: Clone>(&self, product_type: &T) -> *mut T {
        let type_info_layout = Layout::new::<T>();
        let memory = alloc::alloc_zeroed(type_info_layout);
        let type_info_ptr = memory as *mut T;
        type_info_ptr.write(product_type.clone());
        type_info_ptr
    }

    // noinspection ALL
    #[allow(clippy::type_complexity)]
    pub unsafe fn read_obj(&self, p: *const usize) -> Result<(Box<dyn Any>, Box<dyn TypeInfo>), AllocatorError> {
        let first_byte = *p;
        match first_byte {
            _ if first_byte == TypeSig::INT => {
                let p_type_info = p.add(1);
                let int_type = *p_type_info as *const IntType;
                let p_value = p_type_info.add(1) as *const i64;
                Ok((Box::new(*p_value), Box::new(*int_type)))
            },
            _ if first_byte == TypeSig::NAT => {
                let p_type_info = p.add(1);
                let nat_type = *p_type_info as *const NatType;
                let p_value = p_type_info.add(1) as *const u64;
                Ok((Box::new(*p_value), Box::new(*nat_type)))
            },
            _ if first_byte == TypeSig::DOUBLE => {
                let p_type_info = p.add(1);
                let double_type = *p_type_info as *const DoubleType;
                let p_value = p_type_info.add(1) as *const f64;
                Ok((Box::new(*p_value), Box::new(*double_type)))
            },
            _ if first_byte == TypeSig::CHAR => {
                let p_type_info = p.add(1);
                let char_type = *p_type_info as *const CharType;
                let p_value = p_type_info.add(1) as *const char;
                Ok((Box::new(*p_value), Box::new(*char_type)))
            },
            _ if first_byte == TypeSig::BOOL => {
                let p_type_info = p.add(1);
                let bool_type = *p_type_info as *const BoolType;
                let p_value = p_type_info.add(1) as *const bool;
                Ok((Box::new(*p_value), Box::new(*bool_type)))
            },
            _ if first_byte == TypeSig::PRODUCT => {
                let p_type_info = p.add(1); // type sig
                let product_type = *p_type_info as *const ProductType; // type data
                let fields = &(*product_type).0;
                let alignment_table = (*product_type).alignment_table();
                let res = self.read_product(fields, &alignment_table, p)?;
                Ok((Box::new(res), Box::new((*product_type).clone())))
            },
            _ if first_byte == TypeSig::RECORD => {
                let p_type_info = p.add(1); // type sig
                let record_type = *p_type_info as *const RecordType; // type data
                let fields = &(*record_type).0;
                let alignment_table = (*record_type).alignment_table();
                let mut vec = Vec::<Arc<dyn any::Any>>::new();
                for (name, field) in fields.iter() { // data fields
                    let field_ptr = p.add(2).add(alignment_table[name]);
                    let value: Arc<dyn Any> = match field.kind() {
                        TypeKind::Nat | TypeKind::Reference => Arc::new(*(field_ptr as *const u64)),
                        TypeKind::Int => Arc::new(*(field_ptr as *const i64)),
                        TypeKind::Double => Arc::new(*(field_ptr as *const f64)),
                        TypeKind::Char => Arc::new(*(field_ptr as *const char)),
                        TypeKind::Bool => Arc::new(*(field_ptr as *const bool)),
                        _ => return Err(AllocatorError::ObjectAllocationFailed("Only primitive types are supported in Product Type".to_string()))
                    };
                    vec.push(value);
                }
                Ok((Box::new(vec), Box::new((*record_type).clone())))
            }
            _ if first_byte == TypeSig::SUM => {
                let p_type_info = p.add(1);
                let sum_type = *p_type_info as *const SumType;
                let cases = &(*sum_type).0;
                let selected_case = &(*sum_type).1;
                let product_type = cases.get(selected_case).unwrap();
                let res = self.read_product(&(product_type.0), &product_type.alignment_table(), p)?;
                Ok((Box::new(res), Box::new((*sum_type).clone())))
            }
            _ => panic!("")
        }
    }


    // noinspection all
    unsafe fn read_product(&self, fields: &[Arc<dyn TypeInfo>], alignment: &[usize], p: *const usize) -> Result<Vec<Arc<dyn Any>>, AllocatorError> {
        let mut vec = Vec::<Arc<dyn Any>>::new();
        for (i, field) in fields.iter().enumerate() { // data fields
            let field_ptr = p.add(2).add(alignment[i]);
            let value: Arc<dyn Any> = match field.kind() {
                TypeKind::Nat | TypeKind::Reference => Arc::new(*(field_ptr as *const u64)),
                TypeKind::Int => Arc::new(*(field_ptr as *const i64)),
                TypeKind::Double => Arc::new(*(field_ptr as *const f64)),
                TypeKind::Char => Arc::new(*(field_ptr as *const char)),
                TypeKind::Bool => Arc::new(*(field_ptr as *const bool)),
                _ => return Err(AllocatorError::ObjectAllocationFailed("Only primitive types are supported in Product Type".to_string()))
            };
            vec.push(value);
        }
        Ok(vec)
    }
}