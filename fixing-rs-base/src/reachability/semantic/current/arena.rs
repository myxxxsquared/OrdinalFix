use super::{super::SProcessor, SIterWrap};
use crate::{props::UnionProp, reachability::GProcessor, utils::RefArena};

pub struct SCurrentArena<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc>
where
    PG: UnionProp,
    PSI: UnionProp,
    PSS: UnionProp,
    GProc: GProcessor<PG = PG>,
    SProc: SProcessor<PG = PG, PSI = PSI, PSS = PSS>,
{
    pub(super) sgen: RefArena<SIterWrap<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc>>,
}

impl<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc>
    SCurrentArena<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc>
where
    PG: UnionProp,
    PSI: UnionProp,
    PSS: UnionProp,
    GProc: GProcessor<PG = PG>,
    SProc: SProcessor<PG = PG, PSI = PSI, PSS = PSS>,
{
    pub fn new() -> Self {
        Self {
            sgen: RefArena::new(),
        }
    }
}
