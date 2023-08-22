use std::fmt::Debug;

use crate::{props::UnionProp, reachability::{GKeyRef, Edge}, utils::Pointer};

#[derive(PartialEq, Eq, Hash, Getters, CopyGetters)]
pub struct SKey<'a, 'b, PG, PSI>
where
    PG: UnionProp,
    PSI: UnionProp,
{
    #[getset(get_copy = "pub")]
    syntactic_edge: GKeyRef<'a, 'b, PG>,
    #[getset(get = "pub")]
    inh_prop: PSI,
}

impl<'a, 'b, PG, PSI> SKey<'a, 'b, PG, PSI>
where
    PG: UnionProp,
    PSI: UnionProp,
{
    pub(super) fn new(syntactic_edge: GKeyRef<'a, 'b, PG>, inh_prop: PSI) -> Self {
        Self {
            syntactic_edge,
            inh_prop,
        }
    }
}

impl<PG, PSI> Debug for SKey<'_, '_, PG, PSI>
where
    PG: UnionProp,
    PSI: UnionProp,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SKey")
            .field("begin", &self.syntactic_edge.begin())
            .field("end", &self.syntactic_edge.end())
            .field("symbol", &self.syntactic_edge.symbol().name())
            .field("length", &self.syntactic_edge.length())
            .field("prop", self.syntactic_edge.prop())
            .finish()
    }
}

pub type SKeyRef<'a, 'b, PG, PSI> = Pointer<'b, SKey<'a, 'b, PG, PSI>>;
