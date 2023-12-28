use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

pub trait IterExt<T> {
    fn group_by<K, F>(self, key_selector: F) -> HashMap<K, Vec<T>>
        where K: Eq + Hash,
              F: Fn(&T) -> K;

    fn group_by_sorted<K, F>(self, key_selector: F) -> BTreeMap<K, Vec<T>>
        where K: Eq + Ord,
              F: Fn(&T) -> K;
}

impl <T: Iterator> IterExt<T::Item> for T {
    // noinspection ALL
    fn group_by<K, F>(self, key_selector: F) -> HashMap<K, Vec<T::Item>>
        where K: Eq + Hash,
              F: Fn(&T::Item) -> K {
        let mut map = HashMap::<K, Vec<T::Item>>::new();
        for item in self {
            let key = key_selector(&item);
            let vec = map.entry(key).or_insert_with(Vec::new);
            vec.push(item);
        }
        map
    }

    // noinspection ALL
    fn group_by_sorted<K, F>(self, key_selector: F) -> BTreeMap<K, Vec<T::Item>>
        where K: Eq + Ord,
              F: Fn(&T::Item) -> K {
        let mut map = BTreeMap::<K, Vec<T::Item>>::new();
        for item in self {
            let key = key_selector(&item);
            let vec = map.entry(key).or_insert_with(Vec::new);
            vec.push(item);
        }
        map
    }
}