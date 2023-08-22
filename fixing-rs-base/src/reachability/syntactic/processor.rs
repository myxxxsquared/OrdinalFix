use crate::{
    grammar::SymbolRef,
    props::{IntoPropResult, PropEmpty, PropResult, UnionProp},
};

pub trait GProcessor {
    type PG: UnionProp;
    fn process_non_terminal<'a>(
        &self,
        symbol: SymbolRef<'a>,
        induction_id: usize,
        sub_types: &[Self::PG],
    ) -> PropResult<Self::PG>;
    fn process_symbolic_terminal<'a>(
        &self,
        symbol: SymbolRef<'a>,
        literal: Option<&str>,
    ) -> PropResult<Self::PG>;
}

pub struct SyntacticProcessorEmpty;

impl GProcessor for SyntacticProcessorEmpty {
    type PG = PropEmpty;

    fn process_non_terminal<'a>(
        &self,
        _symbol: SymbolRef<'a>,
        _induction_id: usize,
        _sub_types: &[Self::PG],
    ) -> PropResult<Self::PG> {
        PropEmpty.into_prop_result()
    }

    fn process_symbolic_terminal<'a>(
        &self,
        _symbol: SymbolRef<'a>,
        _literal: Option<&str>,
    ) -> PropResult<Self::PG> {
        PropEmpty.into_prop_result()
    }
}
