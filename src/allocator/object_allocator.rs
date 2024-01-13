use std::{alloc, ptr};
use std::alloc::Layout;
use std::any::Any;
use std::mem::size_of;
use std::sync::Arc;
use linked_hash_map::LinkedHashMap;
use crate::allocator::heap_allocator::HeapAllocator;
use crate::utils::errors::AllocatorError;
use crate::utils::func_ext::OptionExt;
use crate::utils::io::object_size;
use crate::vm_types::type_info::*;
use crate::vm_types::type_kind::TypeKind;
use crate::vm_types::type_sig::TypeSig;
use crate::vm_types::type_tokens;

pub struct ObjectAllocator {
    pub allocator: HeapAllocator,
    pub allocated_objects: Vec<*mut ObjectHeader>
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ObjectHeader {
    pub type_sig: usize,
    pub size: usize,
    pub ptr_to_type_info: *mut dyn TypeInfo
}

impl ObjectHeader {
    pub fn new(type_sig: usize, size: usize, ptr_to_type_info: *mut dyn TypeInfo) -> Self {
        ObjectHeader {
            type_sig,
            size,
            ptr_to_type_info
        }
    }

    pub fn type_sig_within_valid_range(i: usize) -> bool {
        (TypeSig::INT..=TypeSig::SUM).contains(&i)
    }
}

pub trait ObjectHeaderHelper {
    unsafe fn to_data_start<T>(&self) -> *mut T;
}

impl ObjectHeaderHelper for *mut ObjectHeader {
    unsafe fn to_data_start<T>(&self) -> *mut T {
        self.cast::<u8>().add(size_of::<ObjectHeader>()) as *mut T
    }
}

pub static USE_COMPACT_LAYOUT: bool = false;

impl ObjectAllocator {
    pub fn new() -> Self {
        ObjectAllocator {
            allocator: HeapAllocator::new(),
            allocated_objects: Vec::new()
        }
    }

    pub unsafe fn write_int(&mut self, value: i64) -> Result<*mut ObjectHeader, AllocatorError> {
        let size_required = object_size(size_of::<i64>());
        let p = self.allocator.alloc(size_required, size_of::<usize>())? as *mut ObjectHeader;
        p.write(ObjectHeader::new(TypeSig::INT, size_required, &type_tokens::INT as *const IntType as usize as *mut IntType));
        p.to_data_start::<i64>().write(value);
        self.allocated_objects.push(p);
        Ok(p)
    }

    pub unsafe fn write_nat(&mut self, value: u64) -> Result<*mut ObjectHeader, AllocatorError> {
        let size_required = object_size(size_of::<u64>());
        let p = self.allocator.alloc(size_required, size_of::<usize>())? as *mut ObjectHeader;
        p.write(ObjectHeader::new(TypeSig::NAT, size_required, &type_tokens::NAT as *const NatType as usize as *mut IntType));
        p.to_data_start::<u64>().write(value);
        self.allocated_objects.push(p);
        Ok(p)
    }

    pub unsafe fn write_reference(&mut self, value: usize, type_info: &ReferenceType) -> Result<*mut ObjectHeader, AllocatorError> {
        let size_required = object_size(size_of::<u64>());
        let p = self.allocator.alloc(size_required, size_of::<usize>())? as *mut ObjectHeader;
        p.write(ObjectHeader::new(TypeSig::REFERENCE, size_required, self.heap_allocated_type_info(type_info) as *mut dyn TypeInfo));
        p.to_data_start::<usize>().write(value);
        self.allocated_objects.push(p);
        Ok(p)
    }

    pub unsafe fn write_double(&mut self, value: f64) -> Result<*mut ObjectHeader, AllocatorError> {
        let size_required = object_size(size_of::<f64>());
        let p = self.allocator.alloc(size_required, size_of::<usize>())? as *mut ObjectHeader;
        p.write(ObjectHeader::new(TypeSig::DOUBLE, size_required, &type_tokens::DOUBLE as *const DoubleType as usize as *mut IntType));
        p.to_data_start::<f64>().write(value);
        self.allocated_objects.push(p);
        Ok(p)
    }

    pub unsafe fn write_char(&mut self, value: char) -> Result<*mut ObjectHeader, AllocatorError> {
        let size_required = object_size(if USE_COMPACT_LAYOUT { 1 } else { 8 });
        let p = self.allocator.alloc(size_required, size_of::<usize>())?.cast::<ObjectHeader>();
        p.write(ObjectHeader::new(TypeSig::CHAR, size_required, &type_tokens::CHAR as *const CharType as usize as *mut IntType));
        p.to_data_start::<char>().write(value);
        self.allocated_objects.push(p);
        Ok(p)
    }

    pub unsafe fn write_bool(&mut self, value: bool) -> Result<*mut ObjectHeader, AllocatorError> {
        let size_required = object_size(if USE_COMPACT_LAYOUT { 1 } else { 8 });
        let p = self.allocator.alloc(size_required, size_of::<usize>())?.cast::<ObjectHeader>();
        p.write(ObjectHeader::new(TypeSig::BOOL, size_required, &type_tokens::BOOL as *const BoolType as usize as *mut IntType));
        p.to_data_start::<bool>().write(value);
        self.allocated_objects.push(p);
        Ok(p)
    }

    // noinspection ALL
    pub unsafe fn write_record(&mut self, data: &LinkedHashMap<String, Arc<dyn Any>>, type_info: &RecordType) -> Result<*mut ObjectHeader, AllocatorError> {
        let size_required = object_size(type_info.size());
        let p = self.allocator.alloc(size_required, size_of::<usize>())?.cast::<ObjectHeader>();
        p.write(ObjectHeader::new(TypeSig::RECORD, size_required, self.heap_allocated_type_info(type_info) as *mut dyn TypeInfo));
        for (name, field) in type_info.0.iter() {
            let field_ptr = p.to_data_start::<u8>().add(type_info.alignment_table()[name]);
            // NOTE: the cast to u8 is necessary because the pointer arithmetic is done in bytes
            // if this is not done, the pointer arithmetic will be done in the size of the usize,
            // that is, a two byte alignment now becomes a 16 byte alignment. A BIG LEAP FORWARD!
            match field.kind() {
                TypeKind::Nat => {
                    let nat = data[name].downcast_ref::<u64>()
                        .to_result(|| AllocatorError::FailedToReadData(format!("Failed to read data for field {} at {:?}", name, field_ptr)))?;
                    (field_ptr as *mut u64).write(*nat);
                },
                TypeKind::Int => {
                    let int = data[name].downcast_ref::<i64>()
                        .to_result(|| AllocatorError::FailedToReadData(format!("Failed to read data for field {} at {:?}", name, field_ptr)))?;
                    (field_ptr as *mut i64).write(*int);
                },
                TypeKind::Double => {
                    let double = data[name].downcast_ref::<f64>()
                        .to_result(|| AllocatorError::FailedToReadData(format!("Failed to read data for field {} at {:?}", name, field_ptr)))?;
                    (field_ptr as *mut f64).write(*double);
                },
                TypeKind::Char => {
                    let char = data[name].downcast_ref::<char>()
                        .to_result(|| AllocatorError::FailedToReadData(format!("Failed to read data for field {} at {:?}", name, field_ptr)))?;
                    (field_ptr as *mut char).write(*char);
                },
                TypeKind::Bool => {
                    let bool = data[name].downcast_ref::<bool>()
                        .to_result(|| AllocatorError::FailedToReadData(format!("Failed to read data for field {} at {:?}", name, field_ptr)))?;
                    (field_ptr as *mut bool).write(*bool);
                },
                TypeKind::Reference => {
                    let reference = data[name].downcast_ref::<usize>()
                        .to_result(|| AllocatorError::FailedToReadData(format!("Failed to read data for field {} at {:?}", name, field_ptr)))?;
                    (field_ptr as *mut usize).write(*reference);
                },
                _ => return Err(AllocatorError::ObjectAllocationFailed("Only primitive types are supported in Product Type".to_string()))
            }
        }
        self.allocated_objects.push(p);
        Ok(p)
    }

    // noinspection ALL
    pub unsafe fn write_product(&mut self, data: &[Arc<dyn Any>], type_info: &ProductType) -> Result<*mut ObjectHeader, AllocatorError> {
        let size_required = object_size(type_info.size());
        let p = self.allocator.alloc(size_required, size_of::<usize>())?.cast::<ObjectHeader>();
        p.write(ObjectHeader::new(TypeSig::PRODUCT, size_required, self.heap_allocated_type_info(type_info) as *mut dyn TypeInfo));
        self.write_product_data(data, type_info, &type_info.alignment_table(), p.to_data_start())?;
        self.allocated_objects.push(p);
        Ok(p)
    }

    pub unsafe fn write_sum(&mut self, data: &[Arc<dyn Any>], type_info: &SumType) -> Result<*mut ObjectHeader, AllocatorError> {
        let size_required = object_size(type_info.size());
        let p = self.allocator.alloc(size_required, size_of::<usize>())?.cast::<ObjectHeader>();
        p.write(ObjectHeader::new(TypeSig::SUM, size_required, self.heap_allocated_type_info(type_info) as *mut dyn TypeInfo));
        self.write_product_data(data, type_info.0.get(&type_info.1).unwrap(), &type_info.alignment_table(), p.to_data_start())?;
        self.allocated_objects.push(p);
        Ok(p)
    }

    // noinspection all
    unsafe fn write_product_data(&mut self, data: &[Arc<dyn Any>], type_info: &ProductType, alignments: &[usize], data_ptr: *mut u8) -> Result<(), AllocatorError> {
        if data.len() != alignments.len() {
            return Err(AllocatorError::ProductSizeMismatch);
        }
        if data.is_empty() {
            return Ok(());
        }
        for (index, field) in type_info.0.iter().enumerate() {
            let field_ptr = data_ptr.add(alignments[index]);
            match field.kind() {
                TypeKind::Nat => {
                    let nat = data[index].downcast_ref::<u64>()
                        .to_result(|| AllocatorError::FailedToReadData(format!("Failed to read data for {}-th field at {:?}", index, field_ptr)))?;
                    (field_ptr as *mut u64).write(*nat);
                },
                TypeKind::Int => {
                    let int = data[index].downcast_ref::<i64>()
                        .to_result(|| AllocatorError::FailedToReadData(format!("Failed to read data for {}-th field at {:?}", index, field_ptr)))?;
                    (field_ptr as *mut i64).write(*int);
                },
                TypeKind::Double => {
                    let double = data[index].downcast_ref::<f64>()
                        .to_result(|| AllocatorError::FailedToReadData(format!("Failed to read data for {}-th field at {:?}", index, field_ptr)))?;
                    (field_ptr as *mut f64).write(*double);
                },
                TypeKind::Char => {
                    let char = data[index].downcast_ref::<char>()
                        .to_result(|| AllocatorError::FailedToReadData(format!("Failed to read data for {}-th field at {:?}", index, field_ptr)))?;
                    (field_ptr as *mut char).write(*char);
                },
                TypeKind::Bool => {
                    let bool = data[index].downcast_ref::<bool>()
                        .to_result(|| AllocatorError::FailedToReadData(format!("Failed to read data for {}-th field at {:?}", index, field_ptr)))?;
                    (field_ptr as *mut bool).write(*bool);
                },
                TypeKind::Reference => {
                    let reference = data[index].downcast_ref::<usize>()
                        .to_result(|| AllocatorError::FailedToReadData(format!("Failed to read data for {}-th field at {:?}", index, field_ptr)))?;
                    (field_ptr as *mut usize).write(*reference);
                },
                _ => return Err(AllocatorError::ObjectAllocationFailed("Only primitive types are supported in Product Type".to_string()))
            }
        }
        Ok(())
    }

    unsafe fn heap_allocated_type_info<T: TypeInfo + Clone>(&self, product_type: &T) -> *mut T {
        let type_info_layout = Layout::new::<T>();
        let memory = alloc::alloc_zeroed(type_info_layout);
        let type_info_ptr = memory as *mut T;
        type_info_ptr.write(product_type.clone());
        type_info_ptr
    }

    // noinspection ALL
    #[allow(clippy::type_complexity)]
    pub unsafe fn read_obj(&mut self, p: *mut ObjectHeader) -> Result<(Arc<dyn TypeInfo>, Arc<dyn Any>), AllocatorError> {
        let header = &*p;
        match header.type_sig {
            TypeSig::INT =>
                Ok((Arc::new(*(header.ptr_to_type_info.cast::<IntType>())), Arc::new(*p.to_data_start::<i64>()))),
            TypeSig::NAT =>
                Ok((Arc::new(*header.ptr_to_type_info.cast::<NatType>()), Arc::new(*p.to_data_start::<u64>()))),
            TypeSig::REFERENCE =>
                Ok((Arc::new(*header.ptr_to_type_info.cast::<ReferenceType>()), Arc::new(*p.to_data_start::<usize>()))),
            TypeSig::DOUBLE =>
                Ok((Arc::new(*header.ptr_to_type_info.cast::<DoubleType>()), Arc::new(*p.to_data_start::<f64>()))),
            TypeSig::CHAR =>
                Ok((Arc::new(*header.ptr_to_type_info.cast::<CharType>()), Arc::new(*p.to_data_start::<char>()))),
            TypeSig::BOOL =>
                Ok((Arc::new(*header.ptr_to_type_info.cast::<BoolType>()), Arc::new(*p.to_data_start::<bool>()))),
            TypeSig::PRODUCT => {
                let product_type = header.ptr_to_type_info as *const ProductType; // type data
                let fields = &(*product_type).0;
                let alignment_table = (*product_type).alignment_table();
                let res = self.read_product(fields, &alignment_table, p)?;
                Ok((Arc::new((*product_type).clone()), Arc::new(res)))
            },
            TypeSig::RECORD => {
                let record_type = header.ptr_to_type_info as *const RecordType; // type data
                let fields = &(*record_type).0;
                let alignment_table = (*record_type).alignment_table();
                let mut map = LinkedHashMap::<String, Arc<dyn Any>>::new();
                for (name, field) in fields.iter() { // data fields
                    // NOTE: the cast to u8 is necessary because the pointer arithmetic is done in bytes
                    // if this is not done, the pointer arithmetic will be done in the size of the usize,
                    // that is, a two byte alignment now becomes a 16 byte alignment. A BIG LEAP FORWARD!
                    let field_ptr = p.to_data_start::<u8>().add(alignment_table[name]);
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
            TypeSig::SUM => {
                let sum_type = header.ptr_to_type_info as *const SumType;
                let cases = &(*sum_type).0;
                let selected_case = &(*sum_type).1;
                let product_type = cases.get(selected_case).unwrap();
                let res = self.read_product(&(product_type.0), &product_type.alignment_table(), p)?;
                Ok((Arc::new((*sum_type).clone()), Arc::new(res)))
            }
            _ => Err(AllocatorError::ReadObjectFailed(format!("Unknown type signature {}", header.type_sig)))
        }
    }

    // noinspection all
    unsafe fn read_product(&self, fields: &[Arc<dyn TypeInfo>], alignment: &[usize], p: *mut ObjectHeader) -> Result<Vec<Arc<dyn Any>>, AllocatorError> {
        let mut vec = Vec::<Arc<dyn Any>>::new();
        for (i, field) in fields.iter().enumerate() { // data fields
            // NOTE: the cast to u8 is necessary because the pointer arithmetic is done in bytes
            // if this is not done, the pointer arithmetic will be done in the size of the usize,
            // that is, a two byte alignment now becomes a 16 byte alignment. A BIG LEAP FORWARD!
            let field_ptr = p.to_data_start::<u8>().add(alignment[i]);
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

    pub unsafe fn allocate_general(&mut self, tuple: &(Arc<dyn TypeInfo>, Arc<dyn Any>)) -> Result<*mut ObjectHeader, AllocatorError> {
        let (ty, data) = tuple;
        match ty.kind() {
            TypeKind::Nat => self.write_nat(*data.downcast_ref_unchecked::<u64>()),
            TypeKind::Reference => self.write_reference(*data.downcast_ref_unchecked::<usize>(), ty.as_any().downcast_ref_unchecked::<ReferenceType>()),
            TypeKind::Int => self.write_int(*data.downcast_ref_unchecked::<i64>()),
            TypeKind::Double => self.write_double(*data.downcast_ref::<f64>()
                .to_result(|| AllocatorError::FailedToReadData(format!("Failed to allocate data {:?}", data)))?),
            TypeKind::Char => self.write_char(*data.downcast_ref::<char>()
                .to_result(|| AllocatorError::FailedToReadData(format!("Failed to allocate data {:?}", data)))?),
            TypeKind::Bool => self.write_bool(*data.downcast_ref::<bool>()
                .to_result(|| AllocatorError::FailedToReadData(format!("Failed to allocate data {:?}", data)))?),
            TypeKind::Product => {
                let prod = ty.as_any().downcast_ref::<ProductType>()
                    .to_result(|| AllocatorError::FailedToReadData(format!("Failed to reify product type info {:?}", ty.as_any())))?;
                self.write_product(data.downcast_ref::<Vec<Arc<dyn Any>>>()
                                       .to_result(|| AllocatorError::FailedToReadData(format!("Failed to allocate data {:?}", data)))?, prod)
            },
            TypeKind::Record => {
                let record = ty.as_any().downcast_ref::<RecordType>()
                    .to_result(|| AllocatorError::FailedToReadData(format!("Failed to reify record type info {:?}", ty.as_any())))?;
                self.write_record(data.downcast_ref::<LinkedHashMap<String, Arc<dyn Any>>>()
                                      .to_result(|| AllocatorError::FailedToReadData(format!("Failed to allocate data {:?}", data)))?, record)
            },
            TypeKind::Sum => {
                let sum = ty.as_any().downcast_ref::<SumType>()
                    .to_result(|| AllocatorError::FailedToReadData(format!("Failed to reify sum type info {:?}", ty.as_any())))?;
                self.write_sum(data.downcast_ref::<Vec<Arc<dyn Any>>>()
                                   .to_result(|| AllocatorError::FailedToReadData(format!("Failed to allocate data {:?}", data)))?, sum)
            }
        }
    }
}