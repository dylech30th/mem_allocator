use std::collections::HashSet;
use rand::Rng;
use crate::allocator::object_allocator::ObjectHeader;
use crate::gc::reachability::ObjectAllocatorExt;
use crate::test::mocking::ObjectMocker;
use crate::utils::io::format_read_object;

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
    let pointers = obj_mocker.allocator.borrow().heap.pointers_all(&allocated_ptrs).unwrap();
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

    allocated_ptrs.iter().for_each(|x| {
        println!("{}", format_read_object(&obj_mocker.allocator.borrow_mut().heap.read_obj(*x).unwrap()));
    });

    let pointers = obj_mocker.allocator.borrow().heap.pointers_all(&allocated_ptrs).unwrap();
    let mut root_objects = (0..20).map(|_| rand::thread_rng().gen_range(0..1000)).map(|i| allocated_ptrs[i]).collect::<HashSet<*mut ObjectHeader>>().into_iter().collect::<Vec<*mut ObjectHeader>>();
    let reachables = obj_mocker.allocator.borrow().heap.reachable(&root_objects).unwrap();

    obj_mocker.allocator.borrow_mut().mark_living(&mut root_objects);

    let set_bits = obj_mocker.allocator.borrow().all_marked_bits();
    println!("bits met: {:x?}", set_bits);
    println!("Tous les bits met: {}", set_bits.len());
    // println!("Tous les bitmaps sont apparaîtent à pointeurs: {}", reachables.iter()
    //    .filter(|x| !pointers.contains(&(**x as *mut ObjectHeader))).map(|x| *x as *mut ObjectHeader).collect::<Vec<*mut ObjectHeader>>().len());
    println!("Tous les bitmaps sont apparaîtent à reachables: {:?}",reachables.symmetric_difference(&set_bits.into_iter().collect::<_>()).collect::<HashSet<_>>());
    println!("Tous les objets sont bien alignés: {}", obj_mocker.allocator.borrow().heap.allocated_objects.iter().all(|x| *x as usize % 8 == 0));
    println!("Tous les pointeurs sont bien alignés: {}", pointers.iter().all(|x| *x as usize % 8 == 0));
    println!("Objets accéssibilité: {}", reachables.len());
    println!("Tous les objets: {}", pointers.len());
}