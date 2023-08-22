// pub type SIterWrapRef<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc> =

use super::SIterWrapIter;
use crate::{
    props::{PropArray, UnionProp},
    reachability::{FKeyRef, GProcessor, SProcessor, SReachability},
};
use std::slice::Iter;

pub enum SIterOrCache<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc>
where
    PG: UnionProp,
    PSI: UnionProp,
    PSS: UnionProp,
    GProc: GProcessor<PG = PG>,
    SProc: SProcessor<PG = PG, PSI = PSI, PSS = PSS>,
{
    Iter(SIterWrapIter<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc>),
    Cache(Iter<'b, (PropArray<PSS>, FKeyRef<'a, 'b, PG, PSI, PSS>)>),
}

impl<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc>
    SIterOrCache<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc>
where
    PG: UnionProp,
    PSI: UnionProp,
    PSS: UnionProp,
    GProc: GProcessor<PG = PG>,
    SProc: SProcessor<PG = PG, PSI = PSI, PSS = PSS>,
{
    pub(super) fn next(
        &mut self,
        reachability: &mut SReachability<'a, 'b, PG, PSI, PSS>,
    ) -> Option<(PropArray<PSS>, FKeyRef<'a, 'b, PG, PSI, PSS>)> {
        match self {
            SIterOrCache::Iter(ref mut iter) => iter.next(reachability),
            SIterOrCache::Cache(ref mut iter) => iter.next().cloned(),
        }
    }
}
