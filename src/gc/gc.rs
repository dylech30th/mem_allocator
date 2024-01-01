use std::cell::RefCell;
use std::mem::align_of;
use std::rc::Rc;
use crate::allocator::heap_allocator::HeapBlock;
use crate::allocator::object_allocator::{ObjectAllocator, ObjectHeader};
use crate::gc::reachability::ObjectAllocatorExt;
use crate::utils::func_ext::OptionExt;
use crate::utils::io::{count_bits_set, object_size};
use crate::utils::iter_ext::IterExt;
use crate::vm_types::type_info::{ProductType, TypeInfo};

pub struct GarbageCollector {
    pub heap: ObjectAllocator,
    // chaque bit répresente un début d'objet possible
    pub bitmap: Vec<Vec<u8>>
}

struct BitmapIndex {
    bitmap_nth: usize,
    offset: usize,
    bit: usize
}

impl BitmapIndex {
    fn new(bitmap_nth: usize, offset: usize, bit: usize) -> BitmapIndex {
        BitmapIndex {
            bitmap_nth,
            offset,
            bit
        }
    }
}

impl GarbageCollector {
    // le ramasse-miettes, il est alloué sur le tas.
    // mendokusaii...
    pub unsafe fn new() -> Rc<RefCell<GarbageCollector>> {
        let gc = Rc::new(RefCell::new(GarbageCollector {
            heap: ObjectAllocator::new(),
            bitmap: vec![]
        }));

        let cloned = gc.clone();
        gc.clone().borrow_mut().heap.allocator.expand_callback = Box::new(move |tracker| unsafe {
            // bien...
            // chaque bitmap contient les bytes, dans lesquels chaque bit répresente un début d'objet possible
            // puisque les objets sont `align_of::<usize>()`-alignés, alors on divise la taille de la mémoire par
            // `align_of::<usize>()`, de plus, puisque chaque bit répresente un début d'objet, alors on divise
            // de plus par `8`, les bits dans un byte.
            // (*cloned.as_ref().as_ptr()).bitmap.push(vec![0; tracker.size / align_of::<usize>() / 8]);
            (*cloned.as_ref().as_ptr()).bitmap.push(vec![0; tracker.size / align_of::<usize>() / 8]);
        });
        gc
    }

    pub unsafe fn next_object(block: &HeapBlock, this_object_option: Option<*mut ObjectHeader>) -> Option<*mut ObjectHeader> {
        if this_object_option.is_none() {
            // on cherche le premier objet dans le bloc
            return (block.unallocated_start != block.start).then_some(block.start.cast::<ObjectHeader>());
        }

        let this_object = this_object_option.unwrap();
        // désadressage d'un pointeur indirect
        let ty = Box::from_raw((*this_object).ptr_to_type_info.cast::<ProductType>());
        let size = ty.size();
        let obj_end = this_object.cast::<u8>().add(object_size(size));
        let padding = (!(obj_end as usize) + 1) & (align_of::<usize>() - 1);
        let next_object = obj_end.add(padding);
        if next_object < block.unallocated_start {
            // une hypothèse optimiste: les objets sont alloués "compactement",
            // id est, chaque objet est suivi par un autre objet, le seul espace
            // entre eux sera le padding.
            let next_object_header = next_object.cast::<ObjectHeader>();
            // malgré la hypothèse optimiste, on doit vérifier si le type signature est presenté,
            // autrement on rencontrera des erreurs de segmentation et le programme sera panic.
            // c'est noté qu'il y a peut-être des données invalides dans la mémoire ceux qui
            // se trouverent avoir des valeurs valides de type signature. On ne peut pas résoudre
            // ce problème, mais puisque la présence de ramasse-miettes, on l'instruit de compactifier
            // l'espace de la mémoire, afin que notre hypothèse optimiste soit toujours vraie.
            ObjectHeader::type_sig_within_valid_range((*next_object_header).type_sig).then_some(next_object_header)
        } else {
            None
        }
    }

    // C'est un algorithme pour marquer les objets accessibles, il est adopté directement du livre
    // "The Garbage Collection Handbook: The Art of Automatic Memory Management" par Richard Jones et Rafael Lins.
    // Cependant, à la different de l'algorithme dans le livre, notre tas est divisé en plusieurs blocs, pendant ce
    // temps l'algorithme dans le livre ne consière qu'on seul bloc de mémoire. Par conséquent, on doit d'abord
    // logicament traiter le tas comme un seul bloc, soit B1, B2, ..., Bn sont les blocs du tas, il est nécessaire
    // de noter que, malgré que Bn soit alloué après Bk si k < n, il est possible que Bn soit avant Bk dans la mémoire
    // physique, par conséquent, on doit d'abord trouver le premier objet dans le premier bloc, l'algorithme est procédé
    // comme suit:
    // 1. On marque tous les objets dans les racines comme accessibles.
    // 2. On trouve le premier objet dans les racines, noté qu'on devrait trouver le premier bloc logical, puisque on
    //    trouve le premier objet physique dans ce bloc (car les addresses dans un bloc est contiguës)
    // 3. Commencer par le premier objet, on trouve tous les références de l'objet, si la référence est logicalment avant
    //    l'objet courant, on le marque comme accessible et on l'ajoute dans la liste d'attente, sinon, on le marque
    //    simplement
    // 4. Après de trouver tous les références de l'objet courant, on trouve le prochain objet ce qui est marqué, répéter
    //    1 - 3.
    //
    // L'avantage de cet algorithme, c'est qu'il est plutôt vite, la complexité de temps est O(n), id est, il est proportionnel
    // à la taille de la mémoire. De plus, il réquiert moins de mémoire, puisque dans tout moment, la liste d'attent, ce n'est
    // pas grande.
    pub(crate) unsafe fn mark_living(&mut self, gc_roots: &mut [*mut ObjectHeader]) {
        self.reset_all_marks();
        gc_roots.iter().for_each(|root| self.set_marked(*root, true));
        // on trouve le premier objet dans le premier bloc, d'abord on trouve logicalment le premier bloc,
        // puis on trouve le premier objet phisique dans le bloc.
        let first = **gc_roots.iter().group_by_sorted(|root| {
            let block = self.block_of(**root);
            self.index_of_heap_block(block)
        }).first_key_value().unwrap().1.iter().min_by_key(|x| ***x as usize).unwrap();
        self.mark_single(first)
    }

    unsafe fn mark_single(&mut self, first_root_in_block: *mut ObjectHeader) {
        let mut cur = Some(first_root_in_block);
        let mut work_list = Vec::new();
        let end_of_heap = self.heap.allocator.committed_regions.iter().map(|x| x.1.block_end()).max().unwrap();
        while let Some(c) = cur && (c as usize) < end_of_heap as usize {
            work_list.push(cur.unwrap());
            while let Some(ptr) = work_list.pop() {
                for pointer in self.heap.pointers(ptr).unwrap() {
                    let address = pointer as usize;
                    if !pointer.is_null() {
                        self.set_marked(pointer, true);
                        if cur.is_none() {
                            return;
                        }
                        let block_of_ptr = self.block_of(pointer);
                        let block_of_cur = self.block_of(cur.unwrap());
                        // NOTE: il est très important de vérifier non seulement si l'adresse est plus petite que l'adresse courante,
                        // mais aussi si le bloc de l'adresse est plus petit que le bloc courant, car il est possible que l'adresse
                        // est plus grand mais son bloc logicalment est plus avant que le bloc courant.
                        // :( il me faut 2 jours pour trouver ce bug!
                        if (address < cur.unwrap() as usize) || self.index_of_heap_block(block_of_ptr) < self.index_of_heap_block(block_of_cur) {
                            work_list.push(pointer);
                        }
                    }
                }
            }
            cur = self.next_in_bitmap(cur.unwrap());
        }
    }

    fn reset_all_marks(&mut self) {
        self.bitmap.iter_mut().for_each(|vec| vec.fill(0));
    }

    pub fn all_marked_bits(&self) -> Vec<*mut ObjectHeader> {
        let all_indexes = self.bitmap.iter()
            .enumerate()
            .flat_map(|(bitmap_nth, vec)| vec.iter().enumerate().filter(|(_, bit)| **bit != 0).map(|t| (bitmap_nth, t.0)).collect::<Vec<(usize, usize)>>())
            .collect::<Vec<_>>();
        all_indexes.iter().flat_map(|(bitmap_nth, offset)| unsafe {
            let bits = count_bits_set(self.bitmap[*bitmap_nth][*offset]);
            bits.iter().map(move |bit| self.bitmap_index_to_address(BitmapIndex::new(*bitmap_nth, *offset, *bit))).collect::<Vec<_>>()
        }).collect::<Vec<_>>()
    }

    unsafe fn next_in_bitmap(&self, this_object: *mut ObjectHeader) -> Option<*mut ObjectHeader> {
        let BitmapIndex { bitmap_nth, offset, bit } = self.address_to_bitmap_index(this_object);
        // on examine d'abord s'il y a des bits mis dans le même byte
        // on a su qu'il y a au moins un bit mis, ce qui est le bit qui répresente `this_object`
        let this_chunk = count_bits_set(self.bitmap[bitmap_nth][offset]).into_iter().find(|x| *x > bit);
        let res = match this_chunk {
            Some(x) =>
                Some(self.bitmap_index_to_address(BitmapIndex::new(bitmap_nth, offset, x))),
            None => {
                // **x != 0 checks whether there are bits set in the byte
                let bitmap_slice = &self.bitmap[bitmap_nth][offset + 1..];
                let new_index_start_relative = offset + 1;
                let new_index = bitmap_slice.iter().enumerate().find(|(_, x)| **x != 0).map(|o| o.0);
                let bit = new_index.map(|x| *count_bits_set(bitmap_slice[x]).first().unwrap());
                new_index.combine(bit).map(|(new_idx, bit_unwrapped)| self.bitmap_index_to_address(BitmapIndex::new(bitmap_nth, new_index_start_relative + new_idx, bit_unwrapped)))
            }
        };
        // Rechercher le prochain bloc si l'on n'en ai pas trouvé dans le bloc courant
        res.flat_map_none(|| {
            let len = self.bitmap.len();
            (len > bitmap_nth + 1).then(|| (bitmap_nth + 1..len)
                .fold(self.first_in_bitmap(bitmap_nth + 1), |acc, idx| acc.flat_map_none(|| self.first_in_bitmap(idx))))
                .flatten()
        })
    }

    unsafe fn first_in_bitmap(&self, bitmap_nth: usize) -> Option<*mut ObjectHeader> {
        let bitmap = &self.bitmap[bitmap_nth];
        bitmap.iter().enumerate().find(|(_, x)| **x != 0).map(|(i, x)| self.bitmap_index_to_address(BitmapIndex::new(bitmap_nth, i, *count_bits_set(*x).first().unwrap())))
    }

    unsafe fn is_marked(&self, address: *mut ObjectHeader) -> bool {
        let BitmapIndex { bitmap_nth, offset, bit } = self.address_to_bitmap_index(address);
        self.bitmap[bitmap_nth][offset] & (1 << bit) != 0
    }

    unsafe fn set_marked(&mut self, address: *mut ObjectHeader, value: bool) {
        let BitmapIndex { bitmap_nth, offset, bit } = self.address_to_bitmap_index(address);
        if value {
            self.bitmap[bitmap_nth][offset] |= 1 << bit;
        } else {
            self.bitmap[bitmap_nth][offset] &= !(1 << bit);
        }
    }

    unsafe fn block_of(&self, address: *mut ObjectHeader) -> &HeapBlock {
        self.heap.allocator.get_block(address.cast()).unwrap()
    }

    unsafe fn index_of_heap_block(&self, block: &HeapBlock) -> usize {
        self.heap.allocator.block_index(block).unwrap()
    }

    unsafe fn address_to_bitmap_index(&self, address: *mut ObjectHeader) -> BitmapIndex {
        let block = self.heap.allocator.get_block(address.cast()).unwrap();
        let nth = block.relative_offset(address) / align_of::<usize>();
        let offset = nth / 8;
        let bit = nth % 8;
        BitmapIndex {
            bitmap_nth: self.index_of_heap_block(block),
            offset,
            bit
        }
    }

    unsafe fn bitmap_index_to_address(&self, index: BitmapIndex) -> *mut ObjectHeader {
        let BitmapIndex { bitmap_nth, offset, bit } = index;
        // referential equality hack here
        let bitmap_index = self.bitmap.iter().enumerate().find(|(_, vec)| vec.as_ptr() == self.bitmap[bitmap_nth].as_ptr())
            .unwrap().0;
        let block = self.heap.allocator.committed_regions.iter().nth(bitmap_index).unwrap();
        let add_offset = (offset * 8 + bit) * align_of::<usize>();
        block.1.absolute_offset(add_offset)
    }
}