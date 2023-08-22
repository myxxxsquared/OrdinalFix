use crate::{
    containers::Map,
    grammar::{GrammarRuleRef, SymbolRef},
    props::{PropArray, UnionProp},
    utils::Pointer,
};

#[derive(Debug, PartialEq, Eq, Hash, Getters, CopyGetters)]
pub struct FKey<'a, PG, PSI, PSS>
where
    PG: UnionProp,
    PSI: UnionProp,
    PSS: UnionProp,
{
    #[getset(get_copy = "pub")]
    begin: usize,
    #[getset(get_copy = "pub")]
    end: usize,
    #[getset(get_copy = "pub")]
    symbol: SymbolRef<'a>,
    #[getset(get = "pub")]
    gprop: PropArray<PG>,
    #[getset(get = "pub")]
    inh_prop: PSI,
    #[getset(get = "pub")]
    syn_prop: PropArray<PSS>,
}

impl<'a, PG, PSI, PSS> FKey<'a, PG, PSI, PSS>
where
    PG: UnionProp,
    PSI: UnionProp,
    PSS: UnionProp,
{
    pub(super) fn new(
        begin: usize,
        end: usize,
        symbol: SymbolRef<'a>,
        gprop: PropArray<PG>,
        inh_prop: PSI,
        syn_prop: PropArray<PSS>,
    ) -> Self {
        Self {
            begin,
            end,
            symbol,
            gprop,
            inh_prop,
            syn_prop,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, CopyGetters)]
pub struct FRule<'a, 'b, PG, PSI, PSS>
where
    PG: UnionProp,
    PSI: UnionProp,
    PSS: UnionProp,
{
    #[getset(get_copy = "pub")]
    right1: Option<FKeyRef<'a, 'b, PG, PSI, PSS>>,
    #[getset(get_copy = "pub")]
    right2: Option<FKeyRef<'a, 'b, PG, PSI, PSS>>,
    #[getset(get_copy = "pub")]
    grule: GrammarRuleRef<'a>,
}

impl<'a, 'b, PG, PSI, PSS> FRule<'a, 'b, PG, PSI, PSS>
where
    PG: UnionProp,
    PSI: UnionProp,
    PSS: UnionProp,
{
    pub(super) fn new(
        right1: Option<FKeyRef<'a, 'b, PG, PSI, PSS>>,
        right2: Option<FKeyRef<'a, 'b, PG, PSI, PSS>>,
        grule: GrammarRuleRef<'a>,
    ) -> Self {
        Self {
            right1,
            right2,
            grule,
        }
    }
}

#[derive(Debug, CopyGetters, Getters)]
pub struct FEntity<'a, 'b, PG, PSI, PSS>
where
    PG: UnionProp,
    PSI: UnionProp,
    PSS: UnionProp,
{
    #[getset(get_copy = "pub")]
    key: FKeyRef<'a, 'b, PG, PSI, PSS>,
    #[getset(get_copy = "pub")]
    length: usize,
    #[getset(get = "pub")]
    rules: Map<&'b FRule<'a, 'b, PG, PSI, PSS>, FRuleRef<'a, 'b, PG, PSI, PSS>>,
    #[getset(get = "pub")]
    literal: Option<String>,
}

impl<'a, 'b, PG, PSI, PSS> FEntity<'a, 'b, PG, PSI, PSS>
where
    PG: UnionProp,
    PSI: UnionProp,
    PSS: UnionProp,
{
    pub(super) fn new(
        key: FKeyRef<'a, 'b, PG, PSI, PSS>,
        length: usize,
        literal: Option<&str>,
    ) -> Self {
        Self {
            key,
            length,
            rules: Map::new(),
            literal: literal.map(|s| s.to_string()),
        }
    }
    pub(super) fn set_length(&mut self, length: usize) {
        self.length = length;
    }
    pub(super) fn insert_rule(&mut self, rule: FRuleRef<'a, 'b, PG, PSI, PSS>) {
        self.rules.insert(rule.ptr(), rule);
    }
    pub(super) fn clear_rules(&mut self) {
        self.rules.clear();
    }
}

pub type FKeyRef<'a, 'b, PG, PSI, PSS> = Pointer<'b, FKey<'a, PG, PSI, PSS>>;
pub type FRuleRef<'a, 'b, PG, PSI, PSS> = Pointer<'b, FRule<'a, 'b, PG, PSI, PSS>>;
pub type FEntityRef<'a, 'b, PG, PSI, PSS> = Pointer<'b, FEntity<'a, 'b, PG, PSI, PSS>>;
