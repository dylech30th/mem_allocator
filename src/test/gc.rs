use crate::gc::utils::ObjectAllocatorExt;
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
    let pointers = allocated_ptrs.iter().flat_map(|x| {
        let list = obj_mocker.allocator.pointers(*x).unwrap();
        list.iter().map(|p| **p).collect::<Vec<usize>>()
    }).collect::<Vec<usize>>();

    for pointer in pointers {
        println!("{:?}", pointer);
    }
}