use super::{iter_or_cache::SIterOrCache, SCurrentArena, SIter, SIterWrap, SIterWrapRef};
use crate::{
    containers::Map,
    props::UnionProp,
    reachability::{
        semantic::reachability::SReachabilityEdges, GKeyRef, GProcessor, GReachability, SKey,
        SKeyRef, SProcessor, SReachability, SReachabilityArena,
    },
    utils::RefCellFrom,
};
use std::mem;

#[derive(Getters, CopyGetters)]
pub struct SReachabilityCurrent<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc>
where
    PG: UnionProp,
    PSI: UnionProp,
    PSS: UnionProp,
    GProc: GProcessor<PG = PG>,
    SProc: SProcessor<PG = PG, PSI = PSI, PSS = PSS>,
{
    generators: RefCellFrom<
        Map<
            &'b SKey<'a, 'b, PG, PSI>,
            (
                SKeyRef<'a, 'b, PG, PSI>,
                SIterWrapRef<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc>,
            ),
        >,
        SReachabilityEdges<'a, 'b, PG, PSI, PSS>,
    >,

    #[getset(get_copy = "pub")]
    processor: &'q SProc,
    #[getset(get_copy = "pub")]
    base_arena: &'b SReachabilityArena<'a, 'b, PG, PSI, PSS>,
    #[getset(get_copy = "pub")]
    arena: &'c SCurrentArena<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc>,
    #[getset(get_copy = "pub")]
    greachability: &'c GReachability<'a, 'b, 'p, PG, GProc>,
}

impl<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc>
    SReachabilityCurrent<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc>
where
    PG: UnionProp,
    PSI: UnionProp,
    PSS: UnionProp,
    GProc: GProcessor<PG = PG>,
    SProc: SProcessor<PG = PG, PSI = PSI, PSS = PSS>,
{
    pub(super) fn new(
        processor: &'q SProc,
        base_arena: &'b SReachabilityArena<'a, 'b, PG, PSI, PSS>,
        arena: &'c SCurrentArena<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc>,
        greachability: &'c GReachability<'a, 'b, 'p, PG, GProc>,
    ) -> Self {
        Self {
            generators: RefCellFrom::new(Map::new()),
            processor,
            base_arena,
            greachability,
            arena,
        }
    }

    pub(super) fn query_edge(
        &'c self,
        syntactic_edge: GKeyRef<'a, 'b, PG>,
        inh_prop: PSI,
        reachability: &mut SReachability<'a, 'b, PG, PSI, PSS>,
    ) -> SIterOrCache<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc> {
        let (_, edges, cache) = reachability.split();
        let key = SKey::new(syntactic_edge, inh_prop);
        {
            match cache.query_edge(&key) {
                Some(v) => return SIterOrCache::Cache(v),
                None => {}
            }
        }
        {
            let b = self.generators.borrow(edges);
            let result = b.get(&key);
            match result {
                Some((_, it)) => {
                    return SIterOrCache::Iter(it.iter());
                }
                _ => {}
            }
        }
        let keyref = self.base_arena.sedges.alloc(key);
        let itwrap = self
            .arena
            .sgen
            .alloc(SIterWrap::new(SIter::new(self, keyref)));
        self.generators
            .borrow_mut(edges)
            .insert(keyref.ptr(), (keyref, itwrap));
        SIterOrCache::Iter(itwrap.iter())
    }

    pub(super) fn cache(&self, reachability: &mut SReachability<'a, 'b, PG, PSI, PSS>) {
        let mut gens = Map::new();
        let (_, edges, cache) = reachability.split();
        let gens_b = self.generators.borrow_mut(edges);
        mem::swap(&mut gens, gens_b);
        for (_, (kref, itwarp)) in gens.into_iter() {
            cache.add_cache(kref, itwarp.take_props());
        }
    }
}
