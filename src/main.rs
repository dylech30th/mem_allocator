#![allow(incomplete_features)]
#![feature(ptr_sub_ptr)]
#![feature(downcast_unchecked)]
#![feature(unsized_locals, unsized_fn_params)]
#![feature(box_into_inner)]
#![feature(allocator_api)]
#![feature(iter_collect_into)]
#![feature(let_chains)]

use std::any::Any;
use std::mem::size_of;
use std::ops::Shr;
use std::sync::Arc;
use crate::allocator::object_allocator::ObjectHeader;
use crate::gc::gc::GarbageCollector;
use crate::utils::io::object_size;
use crate::vm_types::type_info::{NatType, ProductType, TypeInfo};
use crate::vm_types::type_tokens;

pub(crate) mod allocator;
pub(crate) mod utils;
pub(crate) mod vm_types;
mod gc;
mod test;

// On a en fait une raison plutôt raisonnable pour ne pas implementer le soi-disant "buddy algorithm"
// D'abord, le buddy algorithm a pour effet de reduire la fragmentation de la mémoire, mais dans un heap
// tous les objets qui on va allouer a pour taille de puissance de 2, donc il n'y a pas de fragmentation
// sauf le rembourrage. Ensuite, le buddy algorithm implemente la reduction de la fragmentation en essayant
// de fusionner les blocs "buddies" après la libération d'un bloc, mais dans un heap, on a le ramasse-miettes,
// spécifiquement on a un ramasse-miettes à "mark-compact", la reduction de fragementation sera
// effectuée dans la phase de fragementation du ramasse-miettes.

fn main() {
    let mut vec_48 = Vec::<Arc<dyn Any>>::new();
    vec_48.push(Arc::new(2048i64));
    vec_48.push(Arc::new(2048i64));
    let mut vec_56 = Vec::<Arc<dyn Any>>::new();
    vec_56.push(Arc::new(2048i64));
    vec_56.push(Arc::new(2048i64));
    vec_56.push(Arc::new(2048i64));
    let mut vec_64 = Vec::<Arc<dyn Any>>::new();
    vec_64.push(Arc::new(2048i64));
    vec_64.push(Arc::new(2048i64));
    vec_64.push(Arc::new(2048i64));
    vec_64.push(Arc::new(2048i64));
    let mut vec_128 = Vec::<Arc<dyn Any>>::new();
    vec_128.push(Arc::new(2048i64));
    vec_128.push(Arc::new(2048i64));
    vec_128.push(Arc::new(2048i64));
    vec_128.push(Arc::new(2048i64));
    vec_128.push(Arc::new(2048i64));
    vec_128.push(Arc::new(2048i64));
    vec_128.push(Arc::new(2048i64));
    vec_128.push(Arc::new(2048i64));
    vec_128.push(Arc::new(2048i64));
    vec_128.push(Arc::new(2048i64));
    vec_128.push(Arc::new(2048i64));
    vec_128.push(Arc::new(2048i64));
    let mut vec_512 = Vec::<Arc<dyn Any>>::new();
    vec_512.push(Arc::new(2048i64));
    vec_512.push(Arc::new(2048i64));
    vec_512.push(Arc::new(2048i64));
    vec_512.push(Arc::new(2048i64));
    vec_512.push(Arc::new(2048i64));
    vec_512.push(Arc::new(2048i64));
    vec_512.push(Arc::new(2048i64));
    vec_512.push(Arc::new(2048i64));
    vec_512.push(Arc::new(2048i64));
    vec_512.push(Arc::new(2048i64));
    vec_512.push(Arc::new(2048i64));
    vec_512.push(Arc::new(2048i64));
    vec_512.push(Arc::new(2048i64));
    vec_512.push(Arc::new(2048i64));
    vec_512.push(Arc::new(2048i64));
    vec_512.push(Arc::new(2048i64));
    vec_512.push(Arc::new(2048i64));
    vec_512.push(Arc::new(2048i64));
    vec_512.push(Arc::new(2048i64));
    vec_512.push(Arc::new(2048i64));
    vec_512.push(Arc::new(2048i64));
    vec_512.push(Arc::new(2048i64));
    vec_512.push(Arc::new(2048i64));
    vec_512.push(Arc::new(2048i64));
    vec_512.push(Arc::new(2048i64));
    vec_512.push(Arc::new(2048i64));
    vec_512.push(Arc::new(2048i64));
    vec_512.push(Arc::new(2048i64));
    vec_512.push(Arc::new(2048i64));
    vec_512.push(Arc::new(2048i64));
    vec_512.push(Arc::new(2048i64));
    vec_512.push(Arc::new(2048i64));

    let OF_40_BYTES: (Arc<dyn TypeInfo>, Arc<dyn Any>) = (Arc::new(type_tokens::INT), Arc::new(2048i64));
    let OF_48_BYTES: (Arc<dyn TypeInfo>, Arc<dyn Any>) = (Arc::new(ProductType(vec![Arc::new(type_tokens::INT), Arc::new(type_tokens::INT)])), Arc::new(vec_48));
    let OF_56_BYTES: (Arc<dyn TypeInfo>, Arc<dyn Any>) = (Arc::new(ProductType(vec![Arc::new(type_tokens::INT), Arc::new(type_tokens::INT), Arc::new(type_tokens::INT)])), Arc::new(vec_56));
    let OF_64_BYTES: (Arc<dyn TypeInfo>, Arc<dyn Any>) = (Arc::new(ProductType(vec![Arc::new(type_tokens::INT), Arc::new(type_tokens::INT), Arc::new(type_tokens::INT), Arc::new(type_tokens::INT)])), Arc::new(vec_64));
    let OF_128_BYTES: (Arc<dyn TypeInfo>, Arc<dyn Any>) = (Arc::new(ProductType(vec![
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT)
    ])), Arc::new(vec_128));
    let OF_512_BYTES: (Arc<dyn TypeInfo>, Arc<dyn Any>) = (Arc::new(ProductType(vec![
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
        Arc::new(type_tokens::INT),
    ])), Arc::new(vec_512));

    unsafe {
        let allocator = GarbageCollector::new();
        let first = allocator.borrow_mut().heap.allocate_general(&OF_56_BYTES).expect("");
        allocator.borrow_mut().heap.allocate_general(&OF_48_BYTES).unwrap();
        let third = allocator.borrow_mut().heap.allocate_general(&OF_64_BYTES).expect("");
        allocator.borrow_mut().heap.allocate_general(&OF_128_BYTES).expect("");
        let fifth = allocator.borrow_mut().heap.allocate_general(&OF_48_BYTES).expect("");
        allocator.borrow_mut().heap.allocate_general(&OF_64_BYTES).expect("");
        let seventh = allocator.borrow_mut().heap.allocate_general(&OF_56_BYTES).expect("asdasdasd");
        let oa = allocator.borrow_mut();
        let first_block = oa.heap.allocator.committed_regions.values().collect::<Vec<_>>().first().unwrap().clone();
        let ptr = allocator.as_ref().as_ptr();
        (*ptr).set_marked(first, true);
        (*ptr).set_marked(third, true);
        (*ptr).set_marked(fifth, true);
        (*ptr).set_marked(seventh, true);
        let bitmap = (*ptr).bitmap.clone();
        let offset = (*ptr).compute_locations(first_block);
        let start = first_block.start;
        println!("{:x?}", (*ptr).new_address_after_compaction(first as *mut u8, &offset, first_block));
        println!("{:x?}", (*ptr).new_address_after_compaction(third as *mut u8, &offset, first_block));
        println!("{:x?}", (*ptr).new_address_after_compaction(fifth as *mut u8, &offset, first_block));
        println!("{:x?}", (*ptr).new_address_after_compaction(seventh as *mut u8, &offset, first_block));
        println!("{:?}", offset);
        println!("{}", first_block.unallocated_start as usize - first_block.start as usize);
        // test::object_allocator_test::test_obj_allocation_stability();
        // test::gc::test_reachability()
    }
}
