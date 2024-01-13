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
use crate::test::gc::{test_pointers, test_reachability};
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

union Union {
    higher: u64,
    lower: u32
}

fn main() {
    let mut u = Union { higher: 0b11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111 };
    unsafe {
        println!("{:#b}", u.lower);
        u.lower = 0;
        println!("{:#b}", u.higher)
    }
    unsafe {
        test_reachability()
    }
}
