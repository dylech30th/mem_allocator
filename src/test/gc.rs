use std::collections::HashSet;
use rand::Rng;
use crate::allocator::object_allocator::ObjectHeader;
use crate::gc::reachability::ObjectAllocatorExt;
use crate::test::mocking::ObjectMocker;

pub unsafe fn test_pointers() {
    let mut obj_mocker = ObjectMocker::new();
    // On sauvagarde deux copies des objets alloués, afin qu'on peut
    // detecter si les allocations suivantes interférent avec les objets
    // qui viennent d'être alloués, s'il est vrai, alors les données de
    // l'objet alloué sera corrompu. Par conséquent, on verra que l'objet
    // "mocked" et l'objet "read" sont différents.
    let mut allocated_ptrs = vec![];
    let mut mock_objects = vec![];
    for _ in 1..=100 {
        let res = obj_mocker.mock_and_allocate_object().unwrap();
        mock_objects.push(res.0.clone());
        allocated_ptrs.push(res.1);
    }

    // Récupérer les pointeurs dont chaque objet alloué contient.
    let pointers = obj_mocker.allocator.pointers_all(&allocated_ptrs).unwrap();
    for pointer in pointers {
        println!("{:?}", pointer);
    }
}

pub unsafe fn test_reachability() {
    let mut obj_mocker = ObjectMocker::new();
    let mut allocated_ptrs = vec![];
    (0..1000).for_each(|_| {
        let res = obj_mocker.mock_and_allocate_object().unwrap();
        allocated_ptrs.push(res.1);
    });

    let pointers = obj_mocker.allocator.pointers_all(&allocated_ptrs).unwrap();
    let root_objects = (0..100).map(|_| rand::thread_rng().gen_range(0..1000)).map(|i| allocated_ptrs[i]).collect::<HashSet<*mut ObjectHeader>>().into_iter().collect::<Vec<*mut ObjectHeader>>();
    let reachables = obj_mocker.allocator.reachable(&root_objects).unwrap();

    println!("Objets accéssibilité: {}", reachables.len());
    println!("Tous les objets: {}", pointers.len());
}