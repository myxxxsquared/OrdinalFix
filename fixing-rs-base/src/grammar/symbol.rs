use crate::utils::{Pointer, RefCellFrom};

use super::{grammar::GrammarRuleRef, Grammar, GrammarRuleLength};
use std::fmt::{Debug, Display, Formatter};

pub type SymbolRef<'a> = Pointer<'a, Symbol<'a>>;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SymbolType {
    LiteralTerminal,
    SymbolicTerminal,
    NonTerminal,
}

impl Display for SymbolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SymbolType::LiteralTerminal => write!(f, "LiteralTerminal"),
            SymbolType::SymbolicTerminal => write!(f, "SymbolicTerminal"),
            SymbolType::NonTerminal => write!(f, "NonTerminal"),
        }
    }
}

pub struct Symbol<'a> {
    id: usize,
    symbol_type: SymbolType,
    name: String,

    entity: RefCellFrom<SymbolEntity<'a>, Grammar<'a>>,
}

pub struct SymbolEntity<'a> {
    rules: Vec<GrammarRuleRef<'a>>,

    ref_one: Vec<GrammarRuleRef<'a>>,
    ref_two_left: Vec<GrammarRuleRef<'a>>,
    ref_two_right: Vec<GrammarRuleRef<'a>>,

    is_multi_valued: bool,
}

impl<'a> Symbol<'a> {
    pub(super) fn new(id: usize, symbol_type: SymbolType, name: &str) -> Self {
        Self {
            id,
            symbol_type,
            name: String::from(name),
            entity: RefCellFrom::new(SymbolEntity {
                rules: Vec::new(),
                ref_one: Vec::new(),
                ref_two_left: Vec::new(),
                ref_two_right: Vec::new(),
                is_multi_valued: false,
            }),
        }
    }

    pub(super) fn add_ref_one(&self, rule: GrammarRuleRef<'a>, grammar: &mut Grammar<'a>) {
        assert!(rule.rule_type().length() == GrammarRuleLength::One);
        assert!(std::ptr::eq(&*rule.right1().unwrap(), self));
        self.entity.borrow_mut(grammar).ref_one.push(rule);
    }
    pub(super) fn add_ref_two_left(&self, rule: GrammarRuleRef<'a>, grammar: &mut Grammar<'a>) {
        assert!(rule.rule_type().length() == GrammarRuleLength::Two);
        assert!(std::ptr::eq(&*rule.right1().unwrap(), self));
        self.entity.borrow_mut(grammar).ref_two_left.push(rule);
    }
    pub(super) fn add_ref_two_right(&self, rule: GrammarRuleRef<'a>, grammar: &mut Grammar<'a>) {
        assert!(rule.rule_type().length() == GrammarRuleLength::Two);
        assert!(std::ptr::eq(&*rule.right2().unwrap(), self));
        self.entity.borrow_mut(grammar).ref_two_right.push(rule);
    }
    pub(super) fn add_rule(&self, rule: GrammarRuleRef<'a>, grammar: &mut Grammar<'a>) {
        self.entity.borrow_mut(grammar).rules.push(rule);
    }
    pub(super) fn set_multi_valued(&self, grammar: &mut Grammar<'a>) {
        self.entity.borrow_mut(grammar).is_multi_valued = true;
    }

    pub fn symbol_id(&self) -> usize {
        self.id
    }
    pub fn symbol_type(&self) -> SymbolType {
        self.symbol_type
    }
    pub fn name<'b>(&'b self) -> &'b str {
        &self.name
    }
    pub fn rules<'b>(&'b self, grammar: &'b Grammar<'a>) -> &'b Vec<GrammarRuleRef<'a>> {
        &self.entity.borrow(grammar).rules
    }
    pub fn ref_one<'b>(&'b self, grammar: &'b Grammar<'a>) -> &'b Vec<GrammarRuleRef<'a>> {
        &self.entity.borrow(grammar).ref_one
    }
    pub fn ref_two_left<'b>(&'b self, grammar: &'b Grammar<'a>) -> &'b Vec<GrammarRuleRef<'a>> {
        &self.entity.borrow(grammar).ref_two_left
    }
    pub fn ref_two_right<'b>(&'b self, grammar: &'b Grammar<'a>) -> &'b Vec<GrammarRuleRef<'a>> {
        &self.entity.borrow(grammar).ref_two_right
    }
    pub fn is_multi_valued(&self, grammar: &Grammar<'a>) -> bool {
        self.entity.borrow(grammar).is_multi_valued
    }

    pub fn fmt_all(&self, grammar: &Grammar<'a>, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self)?;
        for rule in self.rules(grammar).iter() {
            writeln!(f, "  {}", rule)?;
        }
        Ok(())
    }
}

impl Display for Symbol<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Symbol[{} {}]", self.symbol_type, self.name)?;
        Ok(())
    }
}

impl Debug for Symbol<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <Symbol<'_> as Display>::fmt(&self, f)
    }
}

impl<'a> Display for SymbolRef<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ptr())
    }
}
