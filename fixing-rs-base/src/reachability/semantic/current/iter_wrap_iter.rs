use crate::{
    props::{PropArray, UnionProp},
    reachability::{FKeyRef, GProcessor, SProcessor, SReachability},
};

use super::SIterWrapRef;

pub struct SIterWrapIter<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc>
where
    PG: UnionProp,
    PSI: UnionProp,
    PSS: UnionProp,
    GProc: GProcessor<PG = PG>,
    SProc: SProcessor<PG = PG, PSI = PSI, PSS = PSS>,
{
    content: SIterWrapRef<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc>,
    cur_loc: usize,
}

impl<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc>
    SIterWrapIter<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc>
where
    PG: UnionProp,
    PSI: UnionProp,
    PSS: UnionProp,
    GProc: GProcessor<PG = PG>,
    SProc: SProcessor<PG = PG, PSI = PSI, PSS = PSS>,
{
    pub(super) fn new(
        content: SIterWrapRef<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc>,
    ) -> Self {
        Self {
            cur_loc: 0,
            content,
        }
    }
    pub(super) fn next(
        &mut self,
        reachability: &mut SReachability<'a, 'b, PG, PSI, PSS>,
    ) -> Option<(PropArray<PSS>, FKeyRef<'a, 'b, PG, PSI, PSS>)> {
        let mut res = self.content.get_from_vec(self.cur_loc);
        if res.is_none() {
            self.content.next(reachability);
            res = self.content.get_from_vec(self.cur_loc);
        }
        self.cur_loc += 1;
        res
    }
}
