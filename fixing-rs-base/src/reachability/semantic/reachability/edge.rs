use crate::{
    containers::Map,
    grammar::{GrammarRuleLength, SymbolType},
    props::{PropArray, UnionProp},
    reachability::{
        Edge, FEntity, FKey, FKeyRef, FRule, GKeyRef, GRuleRef, SKeyRef, SReachabilityArena,
    },
};

pub struct SReachabilityEdges<'a, 'b, PG, PSI, PSS>
where
    PG: UnionProp,
    PSI: UnionProp,
    PSS: UnionProp,
{
    arena: &'b SReachabilityArena<'a, 'b, PG, PSI, PSS>,
    edges: Map<&'b FKey<'a, PG, PSI, PSS>, FEntity<'a, 'b, PG, PSI, PSS>>,
}

impl<'a, 'b, PG, PSI, PSS> SReachabilityEdges<'a, 'b, PG, PSI, PSS>
where
    PG: UnionProp,
    PSI: UnionProp,
    PSS: UnionProp,
{
    pub(super) fn new(arena: &'b SReachabilityArena<'a, 'b, PG, PSI, PSS>) -> Self {
        Self {
            arena,
            edges: Map::new(),
        }
    }

    pub fn get_entity(
        &self,
        key: &FKey<'a, PG, PSI, PSS>,
    ) -> Option<&FEntity<'a, 'b, PG, PSI, PSS>> {
        self.edges.get(key)
    }

    fn find_or_add_edge<'t>(
        arena: &'b SReachabilityArena<'a, 'b, PG, PSI, PSS>,
        edges: &'t mut Map<&'b FKey<'a, PG, PSI, PSS>, FEntity<'a, 'b, PG, PSI, PSS>>,
        gedge: GKeyRef<'a, 'b, PG>,
        inh: &PSI,
        syn: &PropArray<PSS>,
        literal: Option<&str>,
    ) -> &'t mut FEntity<'a, 'b, PG, PSI, PSS> {
        let edge_key = FKey::new(
            gedge.begin(),
            gedge.end(),
            gedge.symbol(),
            gedge.prop().clone(),
            inh.clone(),
            syn.clone(),
        );
        if !edges.contains_key(&edge_key) {
            let edge_key = arena.fkey.alloc(edge_key);
            let edge_entity = FEntity::new(edge_key, gedge.length(), literal);
            edges.insert(edge_key.ptr(), edge_entity);
            edges.get_mut(edge_key.ptr()).unwrap()
        } else {
            edges.get_mut(&edge_key).unwrap()
        }
    }

    pub(in super::super) fn assembly_result(
        &mut self,
        key: SKeyRef<'a, 'b, PG, PSI>,
        syn: &PropArray<PSS>,
        rule: &Option<GRuleRef<'a, 'b, PG>>,
        left_inh: &Option<PSI>,
        left_syn: &Option<PropArray<PSS>>,
        right_inh: &Option<PSI>,
        right_syn: &Option<PropArray<PSS>>,
        literal: Option<&str>,
    ) -> Option<FKeyRef<'a, 'b, PG, PSI, PSS>> {
        let gedge = key.syntactic_edge();
        let (left, right) = match gedge.symbol().symbol_type() {
            SymbolType::NonTerminal => {
                let rule = rule.unwrap();
                let length = rule.rule().rule_type().length();

                let left = match length {
                    GrammarRuleLength::Zero => None,
                    GrammarRuleLength::One | GrammarRuleLength::Two => Some(
                        Self::find_or_add_edge(
                            self.arena,
                            &mut self.edges,
                            rule.sub1().unwrap(),
                            left_inh.as_ref().unwrap(),
                            left_syn.as_ref().unwrap(),
                            None,
                        )
                        .key(),
                    ),
                };

                let right = match length {
                    GrammarRuleLength::Zero | GrammarRuleLength::One => None,
                    GrammarRuleLength::Two => Some(
                        Self::find_or_add_edge(
                            self.arena,
                            &mut self.edges,
                            rule.sub2().unwrap(),
                            right_inh.as_ref().unwrap(),
                            right_syn.as_ref().unwrap(),
                            None,
                        )
                        .key(),
                    ),
                };

                (left, right)
            }
            _ => (None, None),
        };

        let edge = Self::find_or_add_edge(
            self.arena,
            &mut self.edges,
            gedge.clone(),
            key.inh_prop(),
            syn,
            literal,
        );

        if edge.length() < gedge.length() {
            None
        } else {
            if edge.length() > gedge.length() {
                edge.set_length(gedge.length());
                edge.clear_rules();
            }

            match gedge.symbol().symbol_type() {
                SymbolType::NonTerminal => {
                    let rule = rule.as_ref().unwrap();
                    let frule = FRule::new(left, right, rule.rule());
                    let frule = self.arena.frule.alloc(frule);
                    edge.insert_rule(frule);
                }
                _ => {}
            }
            Some(edge.key())
        }
    }
}
