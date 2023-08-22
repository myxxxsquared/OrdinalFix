use super::{
    syntactic::{GKey, GRule},
    FKey, FRule, SKey, SReachabilityCacheEntity,
};
use crate::{props::UnionProp, utils::RefArena};

pub struct GReachabilityArena<'a, 'b, PG>
where
    PG: UnionProp,
{
    pub(super) gedges: RefArena<GKey<'a, PG>>,
    pub(super) grules: RefArena<GRule<'a, 'b, PG>>,
    pub(super) tokens: RefArena<String>,
}

impl<'a, 'b, PG> GReachabilityArena<'a, 'b, PG>
where
    PG: UnionProp,
{
    pub fn new() -> Self {
        Self {
            gedges: RefArena::new(),
            grules: RefArena::new(),
            tokens: RefArena::new(),
        }
    }
}

pub struct SReachabilityArena<'a, 'b, PG, PSI, PSS>
where
    PG: UnionProp,
    PSI: UnionProp,
    PSS: UnionProp,
{
    pub(super) sedges: RefArena<SKey<'a, 'b, PG, PSI>>,
    pub(super) fkey: RefArena<FKey<'a, PG, PSI, PSS>>,
    pub(super) frule: RefArena<FRule<'a, 'b, PG, PSI, PSS>>,
    pub(super) cache_entity: RefArena<SReachabilityCacheEntity<'a, 'b, PG, PSI, PSS>>,
}

impl<'a, 'b, PG, PSI, PSS> SReachabilityArena<'a, 'b, PG, PSI, PSS>
where
    PG: UnionProp,
    PSI: UnionProp,
    PSS: UnionProp,
{
    pub fn new() -> Self {
        Self {
            sedges: RefArena::new(),
            fkey: RefArena::new(),
            frule: RefArena::new(),
            cache_entity: RefArena::new(),
        }
    }
}
