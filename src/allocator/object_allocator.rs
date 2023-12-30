use std::{alloc, ptr};
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

#[repr(C)]
pub struct ObjectHeader {
    pub type_sig: usize,
    pub ptr_to_type_info: usize
}

pub static USE_COMPACT_LAYOUT: bool = false;

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

    pub unsafe fn write_nat(&mut self, value: u64) -> Result<*mut usize, AllocatorError> {
        let size_required = size_of::<usize>() + size_of::<usize>() + size_of::<u64>();
        let p = self.allocator.alloc(size_required, size_of::<usize>())? as *mut usize;
        p.write(TypeSig::NAT);
        p.add(1).write(&type_tokens::NAT as *const NatType as usize);
        (p.add(2) as *mut u64).write(value);
        self.allocated_objects.push(p);
        Ok(p)
    }

    pub unsafe fn write_reference(&mut self, value: usize, type_info: &ReferenceType) -> Result<*mut usize, AllocatorError> {
        let size_required = size_of::<usize>() + size_of::<usize>() + size_of::<u64>();
        let p = self.allocator.alloc(size_required, size_of::<usize>())? as *mut usize;
        p.write(TypeSig::REFERENCE);
        p.add(1).write(self.heap_allocated_type_info(type_info) as usize);
        p.add(2).write(value);
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
        for (name, field) in type_info.0.iter() {
            let field_ptr = (data_ptr as *mut u8).add(type_info.alignment_table()[name]);
            // NOTE: the cast to u8 is necessary because the pointer arithmetic is done in bytes
            // if this is not done, the pointer arithmetic will be done in the size of the usize,
            // that is, a two byte alignment now becomes a 16 byte alignment. A BIG LEAP FORWARD!
            match field.kind() {
                TypeKind::Nat => {
                    let nat = data[name].downcast_ref::<u64>().unwrap();
                    (field_ptr as *mut u64).write(*nat);
                },
                TypeKind::Int => {
                    let int = data[name].downcast_ref::<i64>().unwrap();
                    (field_ptr as *mut i64).write(*int);
                },
                TypeKind::Double => {
                    let double = data[name].downcast_ref::<f64>().unwrap();
                    (field_ptr as *mut f64).write(*double);
                },
                TypeKind::Char => {
                    let char = data[name].downcast_ref::<char>().unwrap();
                    (field_ptr as *mut char).write(*char);
                },
                TypeKind::Bool => {
                    let bool = data[name].downcast_ref::<bool>().unwrap();
                    (field_ptr as *mut bool).write(*bool);
                },
                TypeKind::Reference => {
                    let reference = data[name].downcast_ref::<usize>().unwrap();
                    (field_ptr as *mut usize).write(*reference);
                },
                _ => return Err(AllocatorError::ObjectAllocationFailed("Only primitive types are supported in Product Type".to_string()))
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
        self.write_product_data(data, type_info, &type_info.alignment_table(), p.add(2))?;
        Ok(p)
    }

    pub unsafe fn write_sum(&mut self, data: &[Arc<dyn Any>], type_info: &SumType) -> Result<*mut usize, AllocatorError> {
        let size_required = size_of::<usize>() + size_of::<usize>() + type_info.size();
        let p = self.allocator.alloc(size_required, size_of::<usize>()).unwrap() as *mut usize;
        p.write(TypeSig::SUM);
        p.add(1).write(self.heap_allocated_type_info(type_info) as usize);
        self.write_product_data(data, type_info.0.get(&type_info.1).unwrap(), &type_info.alignment_table(), p.add(2))?;
        Ok(p)
    }

    // noinspection all
    unsafe fn write_product_data(&mut self, data: &[Arc<dyn Any>], type_info: &ProductType, alignments: &[usize], data_ptr: *mut usize) -> Result<(), AllocatorError> {
        if data.len() != alignments.len() {
            return Err(AllocatorError::ProductSizeMismatch);
        }
        if data.is_empty() {
            return Ok(());
        }
        for (index, field) in type_info.0.iter().enumerate() {
            // NOTE: the cast to u8 is necessary because the pointer arithmetic is done in bytes
            // if this is not done, the pointer arithmetic will be done in the size of the usize,
            // that is, a two byte alignment now becomes a 16 byte alignment. A BIG LEAP FORWARD!
            let field_ptr = (data_ptr as *mut u8).add(alignments[index]);
            match field.kind() {
                TypeKind::Nat => {
                    let nat = data[index].downcast_ref::<u64>().unwrap();
                    (field_ptr as *mut u64).write(*nat);
                },
                TypeKind::Int => {
                    let int = data[index].downcast_ref::<i64>().unwrap();
                    (field_ptr as *mut i64).write(*int);
                },
                TypeKind::Double => {
                    let double = data[index].downcast_ref::<f64>().unwrap();
                    (field_ptr as *mut f64).write(*double);
                },
                TypeKind::Char => {
                    let char = data[index].downcast_ref::<char>().unwrap();
                    (field_ptr as *mut char).write(*char);
                },
                TypeKind::Bool => {
                    let bool = data[index].downcast_ref::<bool>().unwrap();
                    (field_ptr as *mut bool).write(*bool);
                },
                TypeKind::Reference => {
                    let reference = data[index].downcast_ref::<usize>().unwrap();
                    (field_ptr as *mut usize).write(*reference);
                },
                _ => return Err(AllocatorError::ObjectAllocationFailed("Only primitive types are supported in Product Type".to_string()))
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
    pub unsafe fn read_obj(&mut self, p: *mut usize) -> Result<(Arc<dyn TypeInfo>, Arc<dyn Any>), AllocatorError> {
        let first_byte = *p;
        match first_byte {
            _ if first_byte == TypeSig::INT => {
                let p_type_info = p.add(1);
                let int_type = *p_type_info as *const IntType;
                let p_value = p_type_info.add(1) as *const i64;
                Ok((Arc::new(*int_type), Arc::new(*p_value)))
            },
            _ if first_byte == TypeSig::NAT => {
                let p_type_info = p.add(1);
                let nat_type = *p_type_info as *const NatType;
                let p_value = p_type_info.add(1) as *const u64;
                Ok((Arc::new(*nat_type), Arc::new(*p_value)))
            },
            _ if first_byte == TypeSig::REFERENCE => {
                let p_type_info = p.add(1);
                let ref_type = *p_type_info as *const ReferenceType;
                let p_value = p_type_info.add(1) as *const usize;
                Ok((Arc::new(*ref_type), Arc::new(*p_value)))
            },
            _ if first_byte == TypeSig::DOUBLE => {
                let p_type_info = p.add(1);
                let double_type = *p_type_info as *const DoubleType;
                let p_value = p_type_info.add(1) as *const f64;
                Ok((Arc::new(*double_type), Arc::new(*p_value)))
            },
            _ if first_byte == TypeSig::CHAR => {
                let p_type_info = p.add(1);
                let char_type = *p_type_info as *const CharType;
                let p_value = p_type_info.add(1) as *const char;
                Ok((Arc::new(*char_type), Arc::new(*p_value)))
            },
            _ if first_byte == TypeSig::BOOL => {
                let p_type_info = p.add(1);
                let bool_type = *p_type_info as *const BoolType;
                let p_value = p_type_info.add(1) as *const bool;
                Ok((Arc::new(*bool_type), Arc::new(*p_value)))
            },
            _ if first_byte == TypeSig::PRODUCT => {
                let p_type_info = p.add(1); // type sig
                let product_type = *p_type_info as *const ProductType; // type data
                let fields = &(*product_type).0;
                let alignment_table = (*product_type).alignment_table();
                let res = self.read_product(fields, &alignment_table, p)?;
                Ok((Arc::new((*product_type).clone()), Arc::new(res)))
            },
            _ if first_byte == TypeSig::RECORD => {
                let p_type_info = p.add(1); // type sig
                let record_type = *p_type_info as *const RecordType; // type data
                let fields = &(*record_type).0;
                let alignment_table = (*record_type).alignment_table();
                let mut map = LinkedHashMap::<String, Arc<dyn Any>>::new();
                for (name, field) in fields.iter() { // data fields
                    // NOTE: the cast to u8 is necessary because the pointer arithmetic is done in bytes
                    // if this is not done, the pointer arithmetic will be done in the size of the usize,
                    // that is, a two byte alignment now becomes a 16 byte alignment. A BIG LEAP FORWARD!
                    let field_ptr = (p.add(2) as *mut u8).add(alignment_table[name]);
                    let value: Arc<dyn Any> = match field.kind() {
                        TypeKind::Nat => Arc::new(ptr::read_unaligned(field_ptr.cast::<u64>())),
                        TypeKind::Int => Arc::new(ptr::read_unaligned(field_ptr.cast::<i64>())),
                        TypeKind::Double => Arc::new(ptr::read_unaligned(field_ptr.cast::<f64>())),
                        TypeKind::Char => Arc::new(ptr::read_unaligned(field_ptr) as char),
                        TypeKind::Bool => Arc::new(ptr::read_unaligned(field_ptr.cast::<bool>())),
                        TypeKind::Reference => Arc::new(ptr::read_unaligned(field_ptr.cast::<usize>())),
                        _ => return Err(AllocatorError::ObjectAllocationFailed("Only primitive types are supported in Product Type".to_string()))
                    };
                    map.insert(name.clone(), value);
                }
                Ok((Arc::new((*record_type).clone()), Arc::new(map)))
            }
            _ if first_byte == TypeSig::SUM => {
                let p_type_info = p.add(1);
                let sum_type = *p_type_info as *const SumType;
                let cases = &(*sum_type).0;
                let selected_case = &(*sum_type).1;
                let product_type = cases.get(selected_case).unwrap();
                let res = self.read_product(&(product_type.0), &product_type.alignment_table(), p)?;
                Ok((Arc::new((*sum_type).clone()), Arc::new(res)))
            }
            _ => panic!("")
        }
    }

    // noinspection all
    unsafe fn read_product(&self, fields: &[Arc<dyn TypeInfo>], alignment: &[usize], p: *const usize) -> Result<Vec<Arc<dyn Any>>, AllocatorError> {
        let mut vec = Vec::<Arc<dyn Any>>::new();
        for (i, field) in fields.iter().enumerate() { // data fields
            // NOTE: the cast to u8 is necessary because the pointer arithmetic is done in bytes
            // if this is not done, the pointer arithmetic will be done in the size of the usize,
            // that is, a two byte alignment now becomes a 16 byte alignment. A BIG LEAP FORWARD!
            let field_ptr = (p.add(2) as *mut u8).add(alignment[i]);
            let value: Arc<dyn Any> = match field.kind() {
                TypeKind::Nat => Arc::new(ptr::read_unaligned(field_ptr.cast::<u64>())),
                TypeKind::Int => Arc::new(ptr::read_unaligned(field_ptr.cast::<i64>())),
                TypeKind::Double => Arc::new(ptr::read_unaligned(field_ptr.cast::<f64>())),
                TypeKind::Char => Arc::new(ptr::read_unaligned(field_ptr) as char),
                TypeKind::Bool => Arc::new(ptr::read_unaligned(field_ptr.cast::<bool>())),
                TypeKind::Reference => Arc::new(ptr::read_unaligned(field_ptr.cast::<usize>())),
                _ => return Err(AllocatorError::ObjectAllocationFailed("Only primitive types are supported in Product Type".to_string()))
            };
            vec.push(value);
        }
        Ok(vec)
    }

    pub unsafe fn allocate_general(&mut self, tuple: &(Arc<dyn TypeInfo>, Arc<dyn Any>)) -> Result<*mut usize, AllocatorError> {
        let (ty, data) = tuple;
        match ty.kind() {
            TypeKind::Nat => self.write_nat(*data.downcast_ref_unchecked::<u64>()),
            TypeKind::Reference => self.write_reference(*data.downcast_ref_unchecked::<usize>(), ty.as_any().downcast_ref_unchecked::<ReferenceType>()),
            TypeKind::Int => self.write_int(*data.downcast_ref_unchecked::<i64>()),
            TypeKind::Double => self.write_double(*data.downcast_ref::<f64>().unwrap()),
            TypeKind::Char => self.write_char(*data.downcast_ref::<char>().unwrap()),
            TypeKind::Bool => self.write_bool(*data.downcast_ref::<bool>().unwrap()),
            TypeKind::Product => {
                let prod = ty.as_any().downcast_ref::<ProductType>().unwrap();
                self.write_product(data.downcast_ref::<Vec<Arc<dyn Any>>>().unwrap(), prod)
            },
            TypeKind::Record => {
                let record = ty.as_any().downcast_ref::<RecordType>().unwrap();
                self.write_record(data.downcast_ref::<LinkedHashMap<String, Arc<dyn Any>>>().unwrap(), record)
            },
            TypeKind::Sum => {
                let sum = ty.as_any().downcast_ref::<SumType>().unwrap();
                self.write_sum(data.downcast_ref::<Vec<Arc<dyn Any>>>().unwrap(), sum)
            }
        }
    }
}