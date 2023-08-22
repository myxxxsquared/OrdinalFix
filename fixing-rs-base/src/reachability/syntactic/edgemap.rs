use crate::{grammar::SymbolRef, utils::Pointer};
use std::{collections::HashMap, hash::Hash};

pub trait Edge<'a> {
    type OtherType: Hash + Eq + Clone;

    fn begin(&self) -> usize;
    fn end(&self) -> usize;
    fn symbol(&self) -> SymbolRef<'a>;
    fn length(&self) -> usize;
    fn other<'r>(&'r self) -> &Self::OtherType;
}

pub struct EdgeMap<'a, 'b, E, V>
where
    E: Edge<'a>,
{
    slots: Vec<HashMap<SymbolRef<'a>, HashMap<&'b E::OtherType, (Pointer<'b, E>, V)>>>,
    token_length: usize,
    max_length: usize,
}

impl<'a, 'b, E, V> EdgeMap<'a, 'b, E, V>
where
    E: Edge<'a>,
    V: Default,
{
    pub fn insert_default(&mut self, edge: Pointer<'b, E>) {
        let begin = edge.begin();
        let end = edge.end();
        let length = edge.length();
        let symbol = edge.symbol();
        let other = edge.ptr().other();
        let slot_index = self.slot_index(begin, end, length);
        let slot = &mut self.slots[slot_index];
        slot.contains_key(&symbol);
        if !slot.contains_key(&symbol) {
            slot.insert(symbol, HashMap::new());
        }
        let map = slot.get_mut(&symbol).unwrap();
        map.insert(&other, (edge, V::default()));
    }
}

impl<'a, 'b, E, V> EdgeMap<'a, 'b, E, V>
where
    E: Edge<'a>,
{
    pub fn new(token_length: usize, max_length: usize) -> Self {
        let token_length = token_length + 1;
        let max_length = max_length + 1;
        let total_slots = token_length
            .checked_mul(token_length)
            .unwrap()
            .checked_mul(max_length)
            .unwrap();
        let mut slots = Vec::with_capacity(total_slots);
        for _ in 0..total_slots {
            slots.push(HashMap::new());
        }
        Self {
            slots,
            token_length,
            max_length,
        }
    }

    fn slot_index(&self, begin: usize, end: usize, length: usize) -> usize {
        length + self.max_length * (end + self.token_length * begin)
    }

    pub fn insert(&mut self, edge: Pointer<'b, E>, v: V) {
        let begin = edge.begin();
        let end = edge.end();
        let length = edge.length();
        let symbol = edge.symbol();
        let other = edge.ptr().other();
        let slot_index = self.slot_index(begin, end, length);
        let slot = &mut self.slots[slot_index];
        slot.contains_key(&symbol);
        if !slot.contains_key(&symbol) {
            slot.insert(symbol, HashMap::new());
        }
        let map = slot.get_mut(&symbol).unwrap();
        map.insert(&other, (edge, v));
    }

    pub fn contains_key<'c>(&self, edge: &'c E) -> bool
    where
        'b: 'c,
    {
        let begin = edge.begin();
        let end = edge.end();
        let length = edge.length();
        let symbol = edge.symbol();
        let slot_index = self.slot_index(begin, end, length);
        let slot = &self.slots[slot_index];
        if let Some(map) = slot.get(&symbol) {
            map.contains_key(&edge.other())
        } else {
            false
        }
    }

    pub fn get(&self, edge: &E) -> Option<(Pointer<'b, E>, &V)> {
        let begin = edge.begin();
        let end = edge.end();
        let length = edge.length();
        let symbol = edge.symbol();
        let slot_index = self.slot_index(begin, end, length);
        let slot = &self.slots[slot_index];
        let map = slot.get(&symbol);
        match map {
            Some(map) => map.get(edge.other()).map(|(e, v)| (*e, v)),
            None => None,
        }
    }

    pub fn get_mut(&mut self, edge: &E) -> Option<(Pointer<'b, E>, &mut V)> {
        let begin = edge.begin();
        let end = edge.end();
        let length = edge.length();
        let symbol = edge.symbol();
        let slot_index = self.slot_index(begin, end, length);
        let slot = &mut self.slots[slot_index];
        let map = slot.get_mut(&symbol);
        match map {
            Some(map) => map.get_mut(edge.other()).map(|(e, v)| (*e, v)),
            None => None,
        }
    }

    pub fn all_edges(&self) -> Vec<Pointer<'b, E>> {
        let mut edges = Vec::new();
        for slot in &self.slots {
            for map in slot.values() {
                for (e, _) in map.values() {
                    edges.push(*e);
                }
            }
        }
        edges
    }
}
