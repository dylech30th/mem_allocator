use crate::allocator::object_allocator::ObjectAllocator;
use crate::utils::errors::GCError;
use crate::vm_types::type_info::*;
use crate::vm_types::type_kind::TypeKind;
use crate::vm_types::type_sig::TypeSig;

pub trait ObjectAllocatorExt {
    unsafe fn pointers(&self, obj_start: *mut usize) -> Result<Vec<*mut usize>, GCError>;
}

impl ObjectAllocatorExt for ObjectAllocator {
    // Récupérer tous les pointeurs dont les objets alloués contient.
    unsafe fn pointers(&self, obj_start: *mut usize) -> Result<Vec<*mut usize>, GCError> {
        let read_product_pointers = |type_info: &ProductType| -> Vec<*mut usize> {
            type_info.0.iter().enumerate()
                .filter(|(_, x)| x.kind() == TypeKind::Reference)
                .map(|x| x.0)
                .collect::<Vec<usize>>()
                .iter().map(|x| {
                    let alignment_at = *(*type_info).alignment_table().get(*x).unwrap();
                    obj_start.add(2).cast::<u8>().add(alignment_at).cast::<usize>()
                }).collect()
        };
        match *obj_start {
            TypeSig::NAT | TypeSig::INT | TypeSig::DOUBLE | TypeSig::CHAR | TypeSig::BOOL => Ok(vec![]),
            TypeSig::REFERENCE => {
                Ok(vec![obj_start.add(2)])
            },
            TypeSig::PRODUCT => {
                let type_info = *obj_start.add(1) as *const ProductType;
                Ok(read_product_pointers(&*type_info))
            },
            TypeSig::RECORD => {
                let type_info = *obj_start.add(1) as *const RecordType;
                let res = (*type_info).0.iter()
                    .filter(|(_, x)| x.kind() == TypeKind::Reference)
                    .map(|x| x.0.clone())
                    .collect::<Vec<String>>()
                    .iter().map(|x| {
                        let alignment_at = *(*type_info).alignment_table().get(x).unwrap();
                        obj_start.add(2).cast::<u8>().add(alignment_at).cast::<usize>()
                    }).collect();
                Ok(res)
            },
            TypeSig::SUM => {
                let type_info = *obj_start.add(1) as *const SumType;
                let variant = (*type_info).0.get(&(*type_info).1).unwrap().clone();
                Ok(read_product_pointers(variant.as_ref()))
            },
            _ => Err(GCError::FailedToReadObjectAt(obj_start))
        }
    }
}