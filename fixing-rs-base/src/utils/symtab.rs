use crate::{containers::Map, props::Prop};
use std::{
    collections::hash_map::DefaultHasher,
    fmt::Debug,
    hash::{Hash, Hasher},
    rc::Rc,
};

#[derive(Debug)]
pub struct SymTabContent<K, V>
where
    K: Hash + Ord + Clone + Debug,
    V: Hash + Clone + Eq + Debug,
{
    decled_vars: Map<K, V>,
    hash: u64,
}

#[derive(Clone)]
pub struct SymTab<K, V>
where
    K: Hash + Ord + Clone + Debug,
    V: Hash + Clone + Eq + Debug,
{
    content: Rc<SymTabContent<K, V>>,
}

impl<K, V> Debug for SymTab<K, V>
where
    K: Hash + Ord + Clone + Debug,
    V: Hash + Clone + Eq + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SymTab {{")?;
        for (k, v) in self.iter() {
            write!(f, "{:?}: {:?}, ", k, v)?;
        }
        write!(f, "}}")?;

        Ok(())
    }
}

impl<K, V> SymTab<K, V>
where
    K: Hash + Ord + Clone + Debug,
    V: Hash + Clone + Eq + Debug,
{
    pub fn new() -> Self {
        Self::from_map(Map::new())
    }

    pub fn from_map(map: Map<K, V>) -> Self {
        let hash = Self::compute_hash(&map);
        Self {
            content: Rc::new(SymTabContent {
                decled_vars: map,
                hash,
            }),
        }
    }

    fn compute_hash(m: &Map<K, V>) -> u64 {
        let mut hasher = DefaultHasher::new();
        let mut keys = m.keys().collect::<Vec<_>>();
        keys.sort();
        for key in keys {
            let val = m.get(key).unwrap();
            key.hash(&mut hasher);
            val.hash(&mut hasher);
        }
        hasher.finish()
    }

    pub fn extend_checked(
        &self,
        name: K,
        ty: V,
        check: impl FnOnce(Option<&V>) -> bool,
    ) -> Option<Self> {
        let current = self.content.decled_vars.get(&name);
        match check(current) {
            true => Some(self.extend(name, ty)),
            false => None,
        }
    }

    pub fn extend(&self, name: K, ty: V) -> Self {
        let mut new_map = self.content.decled_vars.clone();
        new_map.insert(name, ty);
        Self::from_map(new_map)
    }

    pub fn extend_multiple(&self, name: impl Iterator<Item = K>, ty: V) -> Self {
        let mut new_map = self.content.decled_vars.clone();
        for k in name {
            new_map.insert(k, ty.clone());
        }
        Self::from_map(new_map)
    }

    pub fn get(&self, name: K) -> Option<V> {
        self.content.decled_vars.get(&name).cloned()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.content.decled_vars.iter()
    }
}

impl<K, V> Default for SymTab<K, V>
where
    K: Hash + Ord + Clone + Debug,
    V: Hash + Clone + Eq + Debug,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> PartialEq for SymTab<K, V>
where
    K: Hash + Ord + Clone + Debug,
    V: Hash + Clone + Eq + Debug,
{
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.content, &other.content)
            || (self.content.hash == other.content.hash
                && self.content.decled_vars.len() == other.content.decled_vars.len()
                && self
                    .content
                    .decled_vars
                    .iter()
                    .all(|(k, v)| other.content.decled_vars.get(k).map_or(false, |v2| v == v2)))
    }
}

impl<K, V> Eq for SymTab<K, V>
where
    K: Hash + Ord + Clone + Debug,
    V: Hash + Clone + Eq + Debug,
{
}

impl<K, V> Hash for SymTab<K, V>
where
    K: Hash + Ord + Clone + Debug,
    V: Hash + Clone + Eq + Debug,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.content.hash.hash(state);
    }
}

impl<K, V> Prop for SymTab<K, V>
where
    K: Hash + Ord + Clone + Debug,
    V: Hash + Clone + Eq + Debug,
{
}
