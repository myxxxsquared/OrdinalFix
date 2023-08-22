use crate::grammar::SymbolType;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlternativeNode<'input> {
    pub id: usize,
    pub elements: Vec<Element<'input>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RulesNode<'input> {
    pub sym: &'input str,
    pub types: Vec<&'input str>,
    pub root_symbol: Option<()>,
    pub alternatives: Vec<AlternativeNode<'input>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Element<'input> {
    pub element_type: SymbolType,
    pub element_value: &'input str,
}

impl<'input> Element<'input> {
    pub fn new(element_type: SymbolType, element_value: &'input str) -> Self {
        Self {
            element_type,
            element_value,
        }
    }
}

#[derive(Clone, Debug)]
pub struct GrammarFile<'input> {
    pub rules: Vec<RulesNode<'input>>,
    pub multivalued_symbols: Vec<&'input str>,
    pub annos: Vec<TerminalAnno<'input>>,
}

#[derive(Clone, Debug)]
pub struct TerminalAnno<'input> {
    pub name: &'input str,
    pub types: Vec<&'input str>,
}
