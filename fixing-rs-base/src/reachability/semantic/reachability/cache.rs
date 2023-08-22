use std::slice::Iter;

use super::super::{FKeyRef, SKeyRef};
use crate::{
    containers::Map,
    props::{PropArray, UnionProp},
    reachability::{SKey, SReachabilityArena},
    utils::Pointer,
};

pub type SReachabilityCacheEntity<'a, 'b, PG, PSI, PSS> =
    Vec<(PropArray<PSS>, FKeyRef<'a, 'b, PG, PSI, PSS>)>;
pub type SReachabilityCacheEntityRef<'a, 'b, PG, PSI, PSS> =
    Pointer<'b, SReachabilityCacheEntity<'a, 'b, PG, PSI, PSS>>;

pub struct SReachabilityCache<'a, 'b, PG, PSI, PSS>
where
    PG: UnionProp,
    PSI: UnionProp,
    PSS: UnionProp,
{
    arena: &'b SReachabilityArena<'a, 'b, PG, PSI, PSS>,
    cache: Map<&'b SKey<'a, 'b, PG, PSI>, SReachabilityCacheEntityRef<'a, 'b, PG, PSI, PSS>>,
}

impl<'a, 'b, PG, PSI, PSS> SReachabilityCache<'a, 'b, PG, PSI, PSS>
where
    PG: UnionProp,
    PSI: UnionProp,
    PSS: UnionProp,
{
    pub(super) fn new(arena: &'b SReachabilityArena<'a, 'b, PG, PSI, PSS>) -> Self {
        Self {
            cache: Map::new(),
            arena,
        }
    }

    pub(in super::super) fn add_cache(
        &mut self,
        key: SKeyRef<'a, 'b, PG, PSI>,
        edges: Vec<(PropArray<PSS>, FKeyRef<'a, 'b, PG, PSI, PSS>)>,
    ) {
        let edges = self.arena.cache_entity.alloc(edges);
        self.cache.insert(key.ptr(), edges);
    }

    pub(in super::super) fn query_edge(
        &self,
        key: &SKey<'a, 'b, PG, PSI>,
    ) -> Option<Iter<'b, (PropArray<PSS>, FKeyRef<'a, 'b, PG, PSI, PSS>)>> {
        match self.cache.get(key) {
            Some(v) => Some(v.ptr().iter()),
            None => None,
        }
    }
}
