use super::{SymbolRef, SymbolType};
use std::fmt::Display;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GrammarRuleType {
    Induction,
    ConcatZero,
    ConcatAppend,
    ConcatTwo,
    ConcatOne,
}

impl GrammarRuleType {
    pub fn length(self) -> GrammarRuleLength {
        match self {
            GrammarRuleType::ConcatZero => GrammarRuleLength::Zero,
            GrammarRuleType::Induction | GrammarRuleType::ConcatOne => GrammarRuleLength::One,
            GrammarRuleType::ConcatAppend | GrammarRuleType::ConcatTwo => GrammarRuleLength::Two,
        }
    }
}

impl Display for GrammarRuleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GrammarRuleType::Induction => write!(f, "Induction"),
            GrammarRuleType::ConcatZero => write!(f, "ConcatZero"),
            GrammarRuleType::ConcatAppend => write!(f, "ConcatAppend"),
            GrammarRuleType::ConcatTwo => write!(f, "ConcatTwo"),
            GrammarRuleType::ConcatOne => write!(f, "ConcatOne"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GrammarRuleLength {
    Zero,
    One,
    Two,
}

impl Display for GrammarRuleLength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GrammarRuleLength::Zero => write!(f, "Zero"),
            GrammarRuleLength::One => write!(f, "One"),
            GrammarRuleLength::Two => write!(f, "Two"),
        }
    }
}

#[derive(Debug)]
pub struct GrammarRule<'a> {
    rule_type: GrammarRuleType,
    left: SymbolRef<'a>,
    right1: Option<SymbolRef<'a>>,
    right2: Option<SymbolRef<'a>>,

    induction: SymbolRef<'a>,
    induction_id: usize,
    induction_args: usize,
    induction_location: Option<usize>,
}

impl<'a> GrammarRule<'a> {
    pub(crate) fn new(
        rule_type: GrammarRuleType,
        left: SymbolRef<'a>,
        right1: Option<SymbolRef<'a>>,
        right2: Option<SymbolRef<'a>>,

        induction: SymbolRef<'a>,
        induction_id: usize,
        induction_args: usize,
        induction_location: Option<usize>,
    ) -> Self {
        assert!(left.symbol_type() == SymbolType::NonTerminal);
        match rule_type.length() {
            GrammarRuleLength::Zero => {
                assert!(right1.is_none());
                assert!(right2.is_none());
            }
            GrammarRuleLength::One => {
                assert!(right1.is_some());
                assert!(right2.is_none());
            }
            GrammarRuleLength::Two => {
                assert!(right1.is_some());
                assert!(right2.is_some());
            }
        }
        match rule_type {
            GrammarRuleType::Induction
            | GrammarRuleType::ConcatZero
            | GrammarRuleType::ConcatOne => {
                assert!(induction_location.is_none());
            }
            GrammarRuleType::ConcatTwo => {
                assert!(induction_location.is_some() && induction_location.unwrap() == 1);
            }
            GrammarRuleType::ConcatAppend => {
                assert!(induction_location.is_some() && induction_location.unwrap() > 1);
            }
        }
        Self {
            left,
            right1,
            right2,
            rule_type,
            induction,
            induction_id,
            induction_args,
            induction_location,
        }
    }

    pub fn rule_type(&self) -> GrammarRuleType {
        self.rule_type
    }
    pub fn left(&self) -> SymbolRef<'a> {
        self.left
    }
    pub fn right1(&self) -> Option<SymbolRef<'a>> {
        self.right1
    }
    pub fn right2(&self) -> Option<SymbolRef<'a>> {
        self.right2
    }
    pub fn induction(&self) -> SymbolRef<'a> {
        self.induction
    }
    pub fn induction_id(&self) -> usize {
        self.induction_id
    }
    pub fn induction_args(&self) -> usize {
        self.induction_args
    }
    pub fn induction_location(&self) -> Option<usize> {
        self.induction_location
    }
}

impl<'a> Display for GrammarRule<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {} ->",
            self.rule_type,
            self.rule_type.length(),
            self.left
        )?;
        match self.rule_type.length() {
            GrammarRuleLength::Zero => {
                write!(f, "E")?;
            }
            GrammarRuleLength::One => {
                write!(f, " {}", self.right1.unwrap())?;
            }
            GrammarRuleLength::Two => {
                write!(f, " {} {}", self.right1.unwrap(), self.right2.unwrap())?;
            }
        }
        Ok(())
    }
}
