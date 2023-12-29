use crate::allocator::object_allocator::ObjectAllocator;
use crate::gc::utils::ObjectAllocatorExt;
use crate::test;
use crate::test::object_allocator_test::test_obj_alloc_batch;

pub unsafe fn test_pointers() {
    let mut obj_allocator = ObjectAllocator::new();

    // On sauvagarde deux copies des objets alloués, afin qu'on peut
    // detecter si les allocations suivantes interférent avec les objets
    // qui viennent d'être alloués, s'il est vrai, alors les données de
    // l'objet alloué sera corrompu. Par conséquent, on verra que l'objet
    // "mocked" et l'objet "read" sont différents.
    let mut allocated_ptrs = vec![];
    let mut mock_objects = vec![];
    for _ in 1..=100 {
        let res = test::mocking::mock_object(0, false).unwrap();
        mock_objects.push(res.clone());
        let read = test_obj_alloc_batch(&mut obj_allocator, &res);
        allocated_ptrs.push(read.clone());
    }

    // Récupérer les pointeurs dont chaque objet alloué contient.
    let pointers = allocated_ptrs.iter().flat_map(|x| {
        let list = obj_allocator.pointers(*x).unwrap();
        list.iter().map(|p| **p).collect::<Vec<usize>>()
    }).collect::<Vec<usize>>();

    for pointer in pointers {
        println!("{:?}", pointer);
    }
}