use std::collections::HashMap;
use std::hash::Hash;

pub fn group_by<T, K>(descriptors: T, key_fn: fn(&T::Item) -> K) -> HashMap<K, Vec<T::Item>>
where
    T: IntoIterator,
    T::Item: Clone,
    K: Eq + Hash,
{
    let mut map = HashMap::new();
    for ad in descriptors.into_iter() {
        map.entry(key_fn(&ad))
            .and_modify(|e: &mut Vec<T::Item>| e.push(ad.clone()))
            .or_insert_with(|| vec![ad.clone()]);
    }
    map
}

pub fn reduce_to_max_by_key<K, V, M>(
    map: &HashMap<K, Vec<&V>>,
    val_select_fn: fn(&V) -> &M,
) -> Vec<V>
where
    M: Ord,
    V: Clone,
{
    let mut result = vec![];
    for (_key, value) in map {
        if let Some(&max_value) = value.iter().max_by_key(|v| val_select_fn(&v)) {
            result.push(max_value.clone());
        }
    }
    result
}
