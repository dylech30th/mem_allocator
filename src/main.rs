#![allow(incomplete_features)]
#![feature(ptr_sub_ptr)]
#![feature(downcast_unchecked)]
#![feature(unsized_locals, unsized_fn_params)]
#![feature(box_into_inner)]
#![feature(allocator_api)]
#![feature(iter_collect_into)]
#![feature(let_chains)]

use std::ops::Shr;

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
    unsafe {
        let a = !0i32;
        let c_as_u32: u32 = {
            let bytes = a.to_be_bytes();
            u32::from_be_bytes(bytes)
        };
        let x = (!c_as_u32) << 1;
        println!("{x:#b}");
        // test::object_allocator_test::test_obj_allocation_stability();
        // test::gc::test_reachability()
    }
}
