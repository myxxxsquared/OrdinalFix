use crate::{
    grammar::SymbolRef,
    props::{IntoPropResult, PropArray, PropEmpty, PropResult, UnionProp},
};
use std::marker::PhantomData;

pub trait SProcessor {
    type PG: UnionProp;
    type PSI: UnionProp;
    type PSS: UnionProp;

    fn process_non_terminal_inh(
        &self,
        symbol: SymbolRef<'_>,
        gprop: &PropArray<Self::PG>,
        induction_id: usize,
        induction_loc: usize,
        inh: &Self::PSI,
        sub_types: &[Self::PSS],
    ) -> PropResult<Self::PSI>;

    fn process_non_terminal_syn(
        &self,
        symbol: SymbolRef<'_>,
        gprop: &PropArray<Self::PG>,
        induction_id: usize,
        inh: &Self::PSI,
        sub_types: &[Self::PSS],
    ) -> PropResult<Self::PSS>;

    fn process_symbolic_terminal_syn(
        &self,
        symbol: SymbolRef<'_>,
        gprop: &PropArray<Self::PG>,
        inh: &Self::PSI,
        literal: Option<&str>,
    ) -> PropResult<Self::PSS>;

    fn process_symbolic_terminal_gen(
        &self,
        symbol: SymbolRef<'_>,
        gprop: &PropArray<Self::PG>,
        inh: &Self::PSI,
        syn: &Self::PSS,
        literal: Option<&str>,
    ) -> String;

    fn process_root_inh(&self) -> Self::PSI;
}

pub struct SProcessorEmpty<GProp>
where
    GProp: UnionProp,
{
    _phantom: PhantomData<GProp>,
}

impl<GProp> SProcessorEmpty<GProp>
where
    GProp: UnionProp,
{
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<GProp> SProcessor for SProcessorEmpty<GProp>
where
    GProp: UnionProp,
{
    type PG = GProp;
    type PSI = PropEmpty;
    type PSS = PropEmpty;

    fn process_non_terminal_inh(
        &self,
        _symbol: SymbolRef<'_>,
        _gprop: &PropArray<Self::PG>,
        _induction_id: usize,
        _induction_loc: usize,
        _inh: &Self::PSI,
        _sub_types: &[Self::PSS],
    ) -> PropResult<Self::PSI> {
        <Self::PSI as Default>::default().into_prop_result()
    }

    fn process_non_terminal_syn(
        &self,
        _symbol: SymbolRef<'_>,
        _gprop: &PropArray<Self::PG>,
        _induction_id: usize,
        _inh: &Self::PSI,
        _sub_types: &[Self::PSS],
    ) -> PropResult<Self::PSS> {
        <Self::PSS as Default>::default().into_prop_result()
    }

    fn process_symbolic_terminal_syn(
        &self,
        _symbol: SymbolRef<'_>,
        _gprop: &PropArray<Self::PG>,
        _inh: &Self::PSI,
        _literal: Option<&str>,
    ) -> PropResult<Self::PSS> {
        <Self::PSS as Default>::default().into_prop_result()
    }

    fn process_symbolic_terminal_gen(
        &self,
        _symbol: SymbolRef<'_>,
        _gprop: &PropArray<Self::PG>,
        _inh: &Self::PSI,
        _syn: &Self::PSS,
        literal: Option<&str>,
    ) -> String {
        match literal {
            Some(s) => String::from(s),
            None => String::new(),
        }
    }

    fn process_root_inh(&self) -> Self::PSI {
        <Self::PSS as Default>::default()
    }
}
