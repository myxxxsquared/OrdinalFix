use super::{FKey, SProcessor};
use crate::{grammar::SymbolType, props::UnionProp, reachability::SReachabilityArena};
use log::info;

mod cache;
mod edge;
pub use cache::{SReachabilityCache, SReachabilityCacheEntity, SReachabilityCacheEntityRef};
pub use edge::SReachabilityEdges;

pub struct SReachability<'a, 'b, PG, PSI, PSS>
where
    PG: UnionProp,
    PSI: UnionProp,
    PSS: UnionProp,
{
    arena: &'b SReachabilityArena<'a, 'b, PG, PSI, PSS>,
    edges: SReachabilityEdges<'a, 'b, PG, PSI, PSS>,
    cache: SReachabilityCache<'a, 'b, PG, PSI, PSS>,
}

impl<'a, 'b, 'q, PG, PSI, PSS> SReachability<'a, 'b, PG, PSI, PSS>
where
    PG: UnionProp,
    PSI: UnionProp,
    PSS: UnionProp,
{
    pub fn new(arena: &'b SReachabilityArena<'a, 'b, PG, PSI, PSS>) -> Self {
        Self {
            arena,
            edges: SReachabilityEdges::new(arena),
            cache: SReachabilityCache::new(arena),
        }
    }

    pub(super) fn split(
        &mut self,
    ) -> (
        &'b SReachabilityArena<'a, 'b, PG, PSI, PSS>,
        &mut SReachabilityEdges<'a, 'b, PG, PSI, PSS>,
        &mut SReachabilityCache<'a, 'b, PG, PSI, PSS>,
    ) {
        (self.arena, &mut self.edges, &mut self.cache)
    }

    pub fn generate_from(
        &self,
        start: &FKey<'a, PG, PSI, PSS>,
        proc: &impl SProcessor<PG = PG, PSI = PSI, PSS = PSS>,
        verbose: bool,
    ) -> Vec<String> {
        let mut result = Vec::new();
        self.append(start, proc, &mut result, verbose);
        result
    }

    fn append(
        &self,
        current: &FKey<'a, PG, PSI, PSS>,
        proc: &impl SProcessor<PG = PG, PSI = PSI, PSS = PSS>,
        result: &mut Vec<String>,
        verbose: bool,
    ) {
        let entity = self.edges.get_entity(current).unwrap();
        if verbose {
            info!(
                "Gen: {} {} {} {} {:?} {:?} {:?} {:?}",
                current.begin(),
                current.end(),
                current.symbol().name(),
                entity.length(),
                entity.literal(),
                current.gprop(),
                current.inh_prop(),
                current.syn_prop(),
            );
        }
        match current.symbol().symbol_type() {
            SymbolType::NonTerminal => {
                let entity = entity.rules().values().next().unwrap();
                if let Some(key) = entity.right1() {
                    self.append(key.ptr(), proc, result, verbose);
                }
                if let Some(key) = entity.right2() {
                    self.append(key.ptr(), proc, result, verbose);
                }
            }
            SymbolType::LiteralTerminal => {
                result.push(current.symbol().name().to_string());
            }
            SymbolType::SymbolicTerminal => {
                let gen = proc.process_symbolic_terminal_gen(
                    current.symbol(),
                    current.gprop(),
                    &current.inh_prop(),
                    current.syn_prop().unwrap_single(),
                    entity.literal().as_deref(),
                );
                result.push(gen);
            }
        }
    }
}
