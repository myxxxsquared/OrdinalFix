use super::{iter_or_cache::SIterOrCache, SReachabilityCurrent};
use crate::{
    grammar::{GrammarRuleLength, GrammarRuleType, SymbolType},
    props::{IntoPropResult, PropArray, PropResult, PropResultIter, UnionProp},
    reachability::{
        Edge, FKeyRef, GProcessor, GReachability, GRule, GRuleRef, SKeyRef, SProcessor,
        SReachability,
    },
};
use std::collections::hash_map::Values;

pub struct SIter<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc>
where
    PG: UnionProp,
    PSI: UnionProp,
    PSS: UnionProp,
    GProc: GProcessor<PG = PG>,
    SProc: SProcessor<PG = PG, PSI = PSI, PSS = PSS>,
{
    key: SKeyRef<'a, 'b, PG, PSI>,
    greachability: &'c GReachability<'a, 'b, 'p, PG, GProc>,
    state: ItState,
    processor: &'q SProc,
    reachability: &'c SReachabilityCurrent<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc>,

    rule_it: Option<Values<'c, &'b GRule<'a, 'b, PG>, GRuleRef<'a, 'b, PG>>>,
    rule: Option<GRuleRef<'a, 'b, PG>>,
    left_inh_it: Option<PropResultIter<PSI>>,
    left_inh: Option<PSI>,
    left_syn_it: Option<SIterOrCache<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc>>,
    left_syn: Option<PropArray<PSS>>,
    right_inh_it: Option<PropResultIter<PSI>>,
    right_inh: Option<PSI>,
    right_syn_it: Option<SIterOrCache<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc>>,
    right_syn: Option<PropArray<PSS>>,
    syn_it: Option<PropResultIter<PropArray<PSS>>>,
    syn: Option<PropArray<PSS>>,
    literal: Option<&'b str>,
}

enum ItState {
    Initial,
    IterRule,
    IterLeftInh,
    IterLeftSyn,
    IterRightInh,
    IterRightSyn,
    ReadyIterSyn,
    IterSyn,
    AssemblyResult,
    Done,
}

impl<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc>
    SIter<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc>
where
    PG: UnionProp,
    PSI: UnionProp,
    PSS: UnionProp,
    GProc: GProcessor<PG = PG>,
    SProc: SProcessor<PG = PG, PSI = PSI, PSS = PSS>,
{
    pub(super) fn new(
        reachability: &'c SReachabilityCurrent<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc>,
        key: SKeyRef<'a, 'b, PG, PSI>,
    ) -> Self {
        let greachability = reachability.greachability();
        let state = ItState::Initial;
        let processor = reachability.processor();

        Self {
            key,
            reachability,
            greachability,
            processor,
            state,
            rule_it: None,
            rule: None,
            left_inh_it: None,
            left_inh: None,
            left_syn_it: None,
            left_syn: None,
            right_inh_it: None,
            right_inh: None,
            right_syn_it: None,
            right_syn: None,
            syn_it: None,
            syn: None,
            literal: None,
        }
    }
    fn process_left_inh(&self) -> PropResult<PSI> {
        let rule = self.rule.unwrap();
        let inh_prop = self.key.inh_prop();
        match rule.rule().rule_type() {
            GrammarRuleType::Induction => inh_prop.clone().into_prop_result(),
            GrammarRuleType::ConcatOne | GrammarRuleType::ConcatTwo => {
                let grule = rule.rule();
                self.processor.process_non_terminal_inh(
                    grule.induction(),
                    self.key.syntactic_edge().prop(),
                    grule.induction_id(),
                    0,
                    &inh_prop,
                    &[][..],
                )
            }
            GrammarRuleType::ConcatAppend => inh_prop.clone().into_prop_result(),
            _ => unreachable!(),
        }
    }
    fn process_left_syn(
        &self,
        reachability: &mut SReachability<'a, 'b, PG, PSI, PSS>,
    ) -> SIterOrCache<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc> {
        let sub1 = self.rule.unwrap().sub1().unwrap();
        self.reachability
            .query_edge(sub1, self.left_inh.clone().unwrap(), reachability)
    }
    fn process_right_inh(&self) -> PropResult<PSI> {
        let rule = self.rule.unwrap();
        let inh_prop = self.key.inh_prop();
        let left_syn = self.left_syn.as_ref().unwrap();
        match rule.rule().rule_type() {
            GrammarRuleType::ConcatAppend => {
                let grule = rule.rule();
                self.processor.process_non_terminal_inh(
                    grule.induction(),
                    self.key.syntactic_edge().prop(),
                    grule.induction_id(),
                    grule.induction_location().unwrap(),
                    &inh_prop,
                    left_syn.unwrap_multiple(),
                )
            }
            GrammarRuleType::ConcatTwo => {
                let grule = rule.rule();
                self.processor.process_non_terminal_inh(
                    grule.induction(),
                    self.key.syntactic_edge().prop(),
                    grule.induction_id(),
                    grule.induction_location().unwrap(),
                    &inh_prop,
                    &[left_syn.unwrap_single().clone()],
                )
            }
            _ => unreachable!(),
        }
    }
    fn process_right_syn(
        &self,
        reachability: &mut SReachability<'a, 'b, PG, PSI, PSS>,
    ) -> SIterOrCache<'a, 'b, 'c, 'p, 'q, PG, PSI, PSS, GProc, SProc> {
        self.reachability.query_edge(
            self.rule.unwrap().sub2().unwrap(),
            self.right_inh.clone().unwrap(),
            reachability,
        )
    }
    fn process_syn(&mut self) -> PropResult<PropArray<PSS>> {
        let gedge = self.key.syntactic_edge();
        let symbol = gedge.symbol();
        let gprop = gedge.prop();
        match symbol.symbol_type() {
            SymbolType::LiteralTerminal => {
                PropArray::Single(<PSS as Default>::default()).into_prop_result()
            }
            SymbolType::SymbolicTerminal => (self.processor.process_symbolic_terminal_syn(
                symbol,
                gprop,
                &self.key.inh_prop(),
                if gedge.length() == 0 {
                    self.literal =
                        Some(self.greachability.literals()[self.key.syntactic_edge().begin()]);
                    self.literal
                } else {
                    None
                },
            ))
            .into(),
            SymbolType::NonTerminal => {
                let rule = self.rule.unwrap();
                let grule = rule.rule();
                let symbol = grule.induction();
                let induction_id = grule.induction_id();
                match rule.rule().rule_type() {
                    GrammarRuleType::Induction => self
                        .processor
                        .process_non_terminal_syn(
                            symbol,
                            gprop,
                            induction_id,
                            &self.key.inh_prop(),
                            self.left_syn.as_ref().unwrap().unwrap_multiple(),
                        )
                        .into(),
                    GrammarRuleType::ConcatZero => {
                        PropArray::Multiple(vec![].into()).into_prop_result()
                    }
                    GrammarRuleType::ConcatOne => {
                        let left_syn = self.left_syn.as_ref().unwrap();
                        PropArray::Multiple(vec![left_syn.unwrap_single().clone()].into())
                            .into_prop_result()
                    }
                    GrammarRuleType::ConcatTwo => {
                        let left_syn = self.left_syn.as_ref().unwrap();
                        let right_syn = self.right_syn.as_ref().unwrap();
                        PropArray::Multiple(
                            vec![
                                left_syn.unwrap_single().clone(),
                                right_syn.unwrap_single().clone(),
                            ]
                            .into(),
                        )
                        .into_prop_result()
                    }
                    GrammarRuleType::ConcatAppend => {
                        let left_syn = self.left_syn.as_ref().unwrap();
                        let right_syn = self.right_syn.as_ref().unwrap();
                        left_syn
                            .append(right_syn.unwrap_single().clone())
                            .into_prop_result()
                    }
                }
            }
        }
    }

    pub(super) fn next(
        &mut self,
        reachability: &mut SReachability<'a, 'b, PG, PSI, PSS>,
    ) -> Option<(PropArray<PSS>, FKeyRef<'a, 'b, PG, PSI, PSS>)> {
        loop {
            self.state = match self.state {
                ItState::Initial => match self.key.syntactic_edge().symbol().symbol_type() {
                    SymbolType::LiteralTerminal | SymbolType::SymbolicTerminal => {
                        ItState::ReadyIterSyn
                    }
                    SymbolType::NonTerminal => {
                        self.rule_it = Some(
                            self.greachability
                                .get_sub_edges(self.key.syntactic_edge())
                                .values(),
                        );
                        ItState::IterRule
                    }
                },
                ItState::IterRule => match self.rule_it.as_mut().unwrap().next() {
                    Some(rule) => {
                        self.rule = Some(rule.clone());
                        match rule.rule().rule_type().length() {
                            GrammarRuleLength::Zero => ItState::ReadyIterSyn,
                            GrammarRuleLength::One | GrammarRuleLength::Two => {
                                self.left_inh_it = Some(self.process_left_inh().into_iter());
                                ItState::IterLeftInh
                            }
                        }
                    }
                    None => {
                        self.rule.take();
                        ItState::Done
                    }
                },
                ItState::IterLeftInh => match self.left_inh_it.as_mut().unwrap().next() {
                    Some(left_inh) => {
                        self.left_inh = Some(left_inh);
                        self.left_syn_it = Some(self.process_left_syn(reachability));
                        ItState::IterLeftSyn
                    }
                    None => {
                        self.left_inh.take();
                        ItState::IterRule
                    }
                },
                ItState::IterLeftSyn => {
                    match self.left_syn_it.as_mut().unwrap().next(reachability) {
                        Some(left_syn) => {
                            self.left_syn = Some(left_syn.0);
                            match self.rule.unwrap().rule().rule_type().length() {
                                GrammarRuleLength::One => ItState::ReadyIterSyn,
                                GrammarRuleLength::Two => {
                                    self.right_inh_it = Some(self.process_right_inh().into_iter());
                                    ItState::IterRightInh
                                }
                                _ => unreachable!(),
                            }
                        }
                        None => {
                            self.left_syn.take();
                            ItState::IterLeftInh
                        }
                    }
                }
                ItState::IterRightInh => match self.right_inh_it.as_mut().unwrap().next() {
                    Some(right_inh) => {
                        self.right_inh = Some(right_inh);
                        self.right_syn_it = Some(self.process_right_syn(reachability));
                        ItState::IterRightSyn
                    }
                    None => {
                        self.right_inh.take();
                        ItState::IterLeftSyn
                    }
                },
                ItState::IterRightSyn => {
                    match self.right_syn_it.as_mut().unwrap().next(reachability) {
                        Some(right_syn) => {
                            self.right_syn = Some(right_syn.0);
                            ItState::ReadyIterSyn
                        }
                        None => {
                            self.right_syn.take();
                            ItState::IterRightInh
                        }
                    }
                }
                ItState::ReadyIterSyn => {
                    self.syn_it = Some(self.process_syn().into_iter());
                    ItState::IterSyn
                }
                ItState::IterSyn => match self.syn_it.as_mut().unwrap().next() {
                    Some(syn) => {
                        self.syn = Some(syn);
                        ItState::AssemblyResult
                    }
                    None => {
                        self.syn_it.take();
                        self.literal.take();
                        match self.key.syntactic_edge().symbol().symbol_type() {
                            SymbolType::LiteralTerminal | SymbolType::SymbolicTerminal => {
                                ItState::Done
                            }
                            SymbolType::NonTerminal => {
                                match self.rule.as_mut().unwrap().rule().rule_type().length() {
                                    GrammarRuleLength::Zero => ItState::IterRule,
                                    GrammarRuleLength::One => ItState::IterLeftSyn,
                                    GrammarRuleLength::Two => ItState::IterRightSyn,
                                }
                            }
                        }
                    }
                },
                ItState::AssemblyResult => {
                    self.state = ItState::IterSyn;
                    let (_, edges, _) = reachability.split();
                    match edges.assembly_result(
                        self.key,
                        self.syn.as_mut().unwrap(),
                        &self.rule,
                        &self.left_inh,
                        &self.left_syn,
                        &self.right_inh,
                        &self.right_syn,
                        self.literal,
                    ) {
                        Some(fkey) => {
                            break Some((self.syn.take().unwrap(), fkey));
                        }
                        None => continue,
                    }
                }
                ItState::Done => break None,
            }
        }
    }
}
