use super::{SIter, SIterWrapIter};
use crate::{
    props::{PropArray, UnionProp},
    reachability::{FKeyRef, GProcessor, SProcessor, SReachability},
    utils::Pointer,
};
use std::{cell::RefCell, mem};

pub type SIterWrapRef<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc> =
    Pointer<'c, SIterWrap<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc>>;

impl<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc>
    SIterWrapRef<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc>
where
    PG: UnionProp,
    PSI: UnionProp,
    PSS: UnionProp,
    GProc: GProcessor<PG = PG>,
    SProc: SProcessor<PG = PG, PSI = PSI, PSS = PSS>,
{
    pub(super) fn iter(&self) -> SIterWrapIter<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc> {
        SIterWrapIter::new(self.clone())
    }
}

pub struct SIterWrap<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc>
where
    PG: UnionProp,
    PSI: UnionProp,
    PSS: UnionProp,
    GProc: GProcessor<PG = PG>,
    SProc: SProcessor<PG = PG, PSI = PSI, PSS = PSS>,
{
    content: RefCell<(
        SIter<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc>,
        Vec<(PropArray<PSS>, FKeyRef<'a, 'b, PG, PSI, PSS>)>,
    )>,
}

impl<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc>
    SIterWrap<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc>
where
    PG: UnionProp,
    PSI: UnionProp,
    PSS: UnionProp,
    GProc: GProcessor<PG = PG>,
    SProc: SProcessor<PG = PG, PSI = PSI, PSS = PSS>,
{
    pub(super) fn next(&self, reachability: &mut SReachability<'a, 'b, PG, PSI, PSS>) {
        let mut b = self.content.try_borrow_mut().expect("SIterWrap reentrance");
        let (ref mut it, ref mut vec) = *b;
        match it.next(reachability) {
            Some(p) => vec.push(p),
            None => {}
        }
    }

    pub(super) fn get_from_vec(
        &self,
        index: usize,
    ) -> Option<(PropArray<PSS>, FKeyRef<'a, 'b, PG, PSI, PSS>)> {
        self.content
            .try_borrow()
            .expect("SIterWrap reentrance")
            .1
            .get(index)
            .cloned()
    }

    pub(super) fn new(sit: SIter<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc>) -> Self {
        Self {
            content: RefCell::new((sit, Vec::new())),
        }
    }

    pub(super) fn take_props(&self) -> Vec<(PropArray<PSS>, FKeyRef<'a, 'b, PG, PSI, PSS>)> {
        let mut result = Vec::new();
        mem::swap(&mut result, &mut self.content.borrow_mut().1);
        result
    }
}
