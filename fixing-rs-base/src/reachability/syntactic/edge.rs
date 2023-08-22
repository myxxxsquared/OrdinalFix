use std::fmt::Display;

use crate::{
    grammar::{GrammarRuleRef, SymbolRef},
    props::{PropArray, UnionProp},
    utils::Pointer,
};

use super::Edge;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct GKey<'a, PG>
where
    PG: UnionProp,
{
    begin: usize,
    end: usize,
    symbol: SymbolRef<'a>,
    length: usize,
    prop: PropArray<PG>,
}

impl<'a, PG> Edge<'a> for GKey<'a, PG>
where
    PG: UnionProp,
{
    type OtherType = PropArray<PG>;

    fn begin(&self) -> usize {
        self.begin
    }

    fn end(&self) -> usize {
        self.end
    }

    fn symbol(&self) -> SymbolRef<'a> {
        self.symbol
    }

    fn length(&self) -> usize {
        self.length
    }

    fn other(&self) -> &Self::OtherType {
        &self.prop
    }
}

impl<'a, PG> GKey<'a, PG>
where
    PG: UnionProp,
{
    pub(crate) fn new(
        begin: usize,
        end: usize,
        symbol: SymbolRef<'a>,
        length: usize,
        prop: PropArray<PG>,
    ) -> Self {
        Self {
            begin,
            end,
            symbol,
            length,
            prop,
        }
    }

    pub fn prop(&self) -> &PropArray<PG> {
        &self.prop
    }
}

impl<'a, PG> Display for GKey<'a, PG>
where
    PG: UnionProp,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}, {}, {}, {}, {:?}]",
            self.begin, self.end, self.symbol, self.length, self.prop
        )
    }
}

#[derive(PartialEq, Eq, Hash)]
pub struct GRule<'a, 'b, PG>
where
    PG: UnionProp,
{
    sub1: Option<GKeyRef<'a, 'b, PG>>,
    sub2: Option<GKeyRef<'a, 'b, PG>>,
    rule: GrammarRuleRef<'a>,
}

impl<'a, 'b, PG> GRule<'a, 'b, PG>
where
    PG: UnionProp,
{
    pub(crate) fn new(
        sub1: Option<GKeyRef<'a, 'b, PG>>,
        sub2: Option<GKeyRef<'a, 'b, PG>>,
        rule: GrammarRuleRef<'a>,
    ) -> Self {
        Self { sub1, sub2, rule }
    }

    pub fn sub1(&self) -> Option<GKeyRef<'a, 'b, PG>> {
        self.sub1
    }
    pub fn sub2(&self) -> Option<GKeyRef<'a, 'b, PG>> {
        self.sub2
    }
    pub fn rule(&self) -> GrammarRuleRef<'a> {
        self.rule
    }
}

impl<'a, 'b, PG> Display for GRule<'a, 'b, PG>
where
    PG: UnionProp,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.sub1, self.sub2) {
            (Some(sub1), Some(sub2)) => {
                write!(f, "<{} -- {}>", sub1.ptr(), sub2.ptr())
            }
            (Some(sub1), None) => write!(f, "<{}>", sub1.ptr()),
            (None, None) => write!(f, "<>"),
            _ => Ok(()),
        }
    }
}

pub type GKeyRef<'a, 'b, PG> = Pointer<'b, GKey<'a, PG>>;
pub type GRuleRef<'a, 'b, PG> = Pointer<'b, GRule<'a, 'b, PG>>;
