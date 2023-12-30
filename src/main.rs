#![allow(incomplete_features)]
#![feature(ptr_sub_ptr)]
#![feature(downcast_unchecked)]
#![feature(unsized_locals, unsized_fn_params)]
#![feature(char_min)]
#![feature(box_into_inner)]
#![feature(allocator_api)]

pub(crate) mod allocator;
pub(crate) mod utils;
pub(crate) mod vm_types;
mod gc;
mod test;

// On a en fait une raison plutôt raisonnable pour ne pas implementer le soi-disant "buddy algorithm"
// D'abord, le buddy algorithm a pour effet de reduire la fragmentation de la mémoire, mais dans un heap
// tous les objets qui on va allouer a pour taille de puissance de 2, donc il n'y a pas de fragmentation
// sauf le rembourrage. Ensuite, le buddy algorithm implemente la reduction de la fragmentation en essayant
// de fusionner les blocs "buddies" après la libération d'un bloc, mais dans un heap, on a le collecteur
// d'ordures, spécifiquement on a un collecteur de "mark-compact", la reduction de fragementation sera
// effectuée dans la phase de fragementation du collecteur d'ordures.

fn main() {
    unsafe {
        test::gc::test_reachability()
    }
}
