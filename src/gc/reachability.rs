use std::collections::{HashSet};
use maplit::hashset;
use crate::allocator::object_allocator::{ObjectAllocator, ObjectHeader, ObjectHeaderHelper};
use crate::utils::errors::GCError;
use crate::vm_types::type_info::*;
use crate::vm_types::type_kind::TypeKind;
use crate::vm_types::type_sig::TypeSig;
use crate::utils::func_ext::FuncExt;

pub trait ObjectAllocatorExt {
    unsafe fn pointers(&self, obj_start: *mut ObjectHeader) -> Result<HashSet<*mut ObjectHeader>, GCError>;

    unsafe fn pointers_all(&self, obj_starts: &[*mut ObjectHeader]) -> Result<HashSet<*mut ObjectHeader>, GCError>;

    unsafe fn reachable(&self, root_objects: &[*mut ObjectHeader]) -> Result<HashSet<*mut ObjectHeader>, GCError>;
}

unsafe fn reachable(allocator: &ObjectAllocator, root_object: *mut ObjectHeader) -> Result<HashSet<*mut ObjectHeader>, GCError> {
    // Calculer le clôture transitif de la relation d'accéssibilité
    // entre les objets alloués.
    let mut reachable = vec![root_object];
    let mut visited = HashSet::new();
    let mut result = HashSet::new();
    result.insert(root_object);
    while let Some(ptr) = reachable.pop() {
        let pointers = allocator.pointers(ptr)?;
        pointers.into_iter().for_each(|x| {
            result.insert(x).ignore();
            if !visited.contains(&x) {
                reachable.push(x);
            } else {
                visited.insert(x);
            }
        });
    }
    Ok(result)
}

impl ObjectAllocatorExt for ObjectAllocator {
    // Récupérer tous les pointeurs dont les objets alloués contient.
    unsafe fn pointers(&self, obj_start: *mut ObjectHeader) -> Result<HashSet<*mut ObjectHeader>, GCError> {
        let read_product_pointers = |type_info: &ProductType| -> HashSet<*mut ObjectHeader> {
            type_info.0.iter().enumerate()
                .filter(|(_, x)| x.kind() == TypeKind::Reference)
                .map(|x| x.0)
                .collect::<Vec<usize>>()
                .iter().map(|x| {
                let alignment_at = *(*type_info).alignment_table().get(*x).unwrap();
                // NOTE: first cast obj_start to u8 and add to alignment then cast to *usize, now we have
                // a pointer that points to the address of the referee, and after that we dereference
                // it to get the referee's address.
                *obj_start.to_data_start::<u8>().add(alignment_at).cast::<usize>() as *mut ObjectHeader
            }).collect()
        };
        let header = &*obj_start;
        match header.type_sig {
            TypeSig::NAT | TypeSig::INT | TypeSig::DOUBLE | TypeSig::CHAR | TypeSig::BOOL => Ok(hashset!{}),
            // NOTE: first cast obj_start to u8 and add to alignment then cast to usize, now we have
            // a pointer that points to the address of the referee, and after that we dereference
            // it to get the referee's address.
            TypeSig::REFERENCE =>
                Ok(hashset!{*obj_start.to_data_start::<usize>() as *mut ObjectHeader}),
            TypeSig::PRODUCT =>
                Ok(read_product_pointers(&*header.ptr_to_type_info.cast::<ProductType>())),
            TypeSig::RECORD => {
                let type_info = &*header.ptr_to_type_info.cast::<RecordType>();
                let res = (*type_info).0.iter()
                    .filter(|(_, x)| x.kind() == TypeKind::Reference)
                    .map(|x| x.0.clone())
                    .collect::<Vec<String>>()
                    .iter().map(|x| {
                    let alignment_at = *(*type_info).alignment_table().get(x).unwrap();
                    // NOTE: first cast obj_start to u8 and add to alignment then cast to usize, now we have
                    // a pointer that points to the address of the referee, and after that we dereference
                    // it to get the referee's address.
                    *obj_start.to_data_start::<u8>().add(alignment_at).cast::<usize>() as *mut ObjectHeader
                }).collect();
                Ok(res)
            },
            TypeSig::SUM => {
                let type_info = header.ptr_to_type_info as *const SumType;
                let variant = (*type_info).0.get(&(*type_info).1).unwrap().clone();
                Ok(read_product_pointers(variant.as_ref()))
            },
            _ => Err(GCError::FailedToReadObjectAt(obj_start as *const usize))
        }
    }

    unsafe fn pointers_all(&self, obj_starts: &[*mut ObjectHeader]) -> Result<HashSet<*mut ObjectHeader>, GCError> {
        obj_starts.iter().fold(self.pointers(obj_starts[0]), |acc, e|
            acc.and_then(|mut acc_vec| self.pointers(*e).map(|new_vec|
                acc_vec.apply::<HashSet<*mut usize>, _>(|av| av.extend(new_vec)).clone())))
            .map(HashSet::from_iter)
    }

    unsafe fn reachable(&self, root_objects: &[*mut ObjectHeader]) -> Result<HashSet<*mut ObjectHeader>, GCError> {
        if root_objects.is_empty() {
            return Ok(HashSet::new());
        }
        root_objects.iter().fold(reachable(self, root_objects[0]), |acc, e|
            acc.and_then(|mut acc_set|
                reachable(self, *e).map(|new|
                    acc_set.apply::<HashSet<*mut usize>, _>(|a|
                        a.extend(new)).clone())))
    }
}