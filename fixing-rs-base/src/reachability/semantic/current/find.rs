use super::{SCurrentArena, SReachabilityCurrent};
use crate::{
    props::UnionProp,
    reachability::{
        FKeyRef, GProcessor, GReachability, SProcessor, SReachability, SReachabilityArena,
    },
};

pub fn find<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc>(
    processor: &'q SProc,
    base_arena: &'b SReachabilityArena<'a, 'b, PG, PSI, PSS>,
    greachability: &'c GReachability<'a, 'b, 'p, PG, GProc>,
    length_from: usize,
    length_to: usize,
    reachability: &'c mut SReachability<'a, 'b, PG, PSI, PSS>,
) -> Option<FKeyRef<'a, 'b, PG, PSI, PSS>>
where
    PG: UnionProp,
    PSI: UnionProp,
    PSS: UnionProp,
    GProc: GProcessor<PG = PG>,
    SProc: SProcessor<PG = PG, PSI = PSI, PSS = PSS>,
{
    let root_prop = processor.process_root_inh();

    let arena = SCurrentArena::new();
    let current = SReachabilityCurrent::new(processor, base_arena, &arena, greachability);
    let result = find_inner(
        greachability,
        length_from,
        length_to,
        root_prop,
        reachability,
        &current,
    );
    current.cache(reachability);
    result
}

pub fn find_inner<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc>(
    greachability: &'c GReachability<'a, 'b, 'p, PG, GProc>,
    length_from: usize,
    length_to: usize,
    root_prop: PSI,
    reachability: &mut SReachability<'a, 'b, PG, PSI, PSS>,
    current: &'c SReachabilityCurrent<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc>,
) -> Option<FKeyRef<'a, 'b, PG, PSI, PSS>>
where
    PG: UnionProp,
    PSI: UnionProp,
    PSS: UnionProp,
    GProc: GProcessor<PG = PG>,
    SProc: SProcessor<PG = PG, PSI = PSI, PSS = PSS>,
{
    for cur_len in length_from..(length_to + 1) {
        match greachability.get_start_edges().get(cur_len) {
            Some(e) => {
                for gkey in e.iter() {
                    let mut it = current.query_edge(gkey.clone(), root_prop.clone(), reachability);
                    match it.next(reachability) {
                        Some((_, fkey)) => return Some(fkey),
                        None => {}
                    }
                }
            }
            None => {}
        }
    }
    None
}
