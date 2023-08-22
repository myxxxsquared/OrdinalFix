use crate::{
    containers::Map,
    parsing::ast::GrammarFile,
    utils::{Pointer, RefArena},
};

use super::{
    super::parsing::{
        ast::{Element, RulesNode},
        parser_grammar::GrammarFileParser,
    },
    GrammarRule, GrammarRuleLength, GrammarRuleType, ParseError, Symbol, SymbolRef, SymbolType,
};
use std::{
    collections::{HashSet, VecDeque},
    fmt::{Display, Formatter, Write},
};

pub type SymbolMap<'a> = Map<&'a str, SymbolRef<'a>>;
pub type GrammarRuleRef<'a> = Pointer<'a, GrammarRule<'a>>;

impl<'a> Display for GrammarRuleRef<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ptr())
    }
}

pub struct Grammar<'a> {
    literal_terimals: SymbolMap<'a>,
    symbolic_terminals: SymbolMap<'a>,
    non_terminals: SymbolMap<'a>,
    unknown_terminal: Option<SymbolRef<'a>>,

    next_symbol_id: usize,

    zero_productions: Vec<GrammarRuleRef<'a>>,
    start_symbol: Option<SymbolRef<'a>>,
    multi_value_symbols: Vec<SymbolRef<'a>>,

    arena: &'a GrammarArena<'a>,
}

#[derive(Clone, Copy)]
pub struct GrammarSymbolsRef<'a> {
    pub literal_terminals: &'a SymbolMap<'a>,
    pub symbolic_terminals: &'a SymbolMap<'a>,
    pub non_terminals: &'a SymbolMap<'a>,
    pub zero_productions: &'a Vec<GrammarRuleRef<'a>>,
    pub start_symbol: SymbolRef<'a>,
    pub unknown_terminal: SymbolRef<'a>,
}

impl<'a> Grammar<'a> {
    fn construct(arena: &'a GrammarArena<'a>) -> Self {
        Self {
            literal_terimals: Map::new(),
            symbolic_terminals: Map::new(),
            non_terminals: Map::new(),
            unknown_terminal: None,

            next_symbol_id: 0,

            zero_productions: Vec::new(),

            start_symbol: None,
            multi_value_symbols: Vec::new(),

            arena,
        }
    }

    fn find_symbol(&self, symbol_type: SymbolType, name: &str) -> Option<SymbolRef<'a>> {
        let m = match symbol_type {
            SymbolType::LiteralTerminal => &self.literal_terimals,
            SymbolType::NonTerminal => &self.non_terminals,
            SymbolType::SymbolicTerminal => &self.symbolic_terminals,
        };
        m.get(name).map(|w| *w)
    }

    fn add_symbol(&mut self, symbol_type: SymbolType, name: &str) -> SymbolRef<'a> {
        let m = match symbol_type {
            SymbolType::LiteralTerminal => &mut self.literal_terimals,
            SymbolType::NonTerminal => &mut self.non_terminals,
            SymbolType::SymbolicTerminal => &mut self.symbolic_terminals,
        };
        if let Some(s) = m.get(name) {
            *s
        } else {
            let new_symbol_id = self.next_symbol_id;
            self.next_symbol_id += 1;
            let s = Symbol::new(new_symbol_id, symbol_type, name);
            let s = self.arena.symbols.alloc(s);
            let name = &s.ptr().name();
            m.insert(name, s);
            s
        }
    }

    fn add_rule(
        &mut self,
        left: SymbolRef<'a>,
        right1: Option<SymbolRef<'a>>,
        right2: Option<SymbolRef<'a>>,
        rule_type: GrammarRuleType,

        induction: SymbolRef<'a>,
        induction_id: usize,
        induction_args: usize,
        induction_location: Option<usize>,
    ) -> GrammarRuleRef<'a> {
        let rule = GrammarRule::new(
            rule_type,
            left,
            right1,
            right2,
            induction,
            induction_id,
            induction_args,
            induction_location,
        );
        let rule = self.arena.rules.alloc(rule);
        left.add_rule(rule, self);
        match rule_type.length() {
            GrammarRuleLength::Zero => {
                self.zero_productions.push(rule);
            }
            GrammarRuleLength::One => {
                let right1 = right1.unwrap();
                right1.add_ref_one(rule, self);
            }
            GrammarRuleLength::Two => {
                let right1 = right1.unwrap();
                let right2 = right2.unwrap();
                right1.add_ref_two_left(rule, self);
                right2.add_ref_two_right(rule, self);
            }
        }
        rule
    }

    fn add_multi_value_symbol(&mut self, symbol: SymbolRef<'a>) {
        if !symbol.is_multi_valued(self) {
            self.multi_value_symbols.push(symbol);
            symbol.set_multi_valued(self);
        } else {
            panic!("Symbol already set to multi valued.");
        }
    }

    fn add_element<'input>(&mut self, element: &'input Element<'input>) -> SymbolRef<'a> {
        self.add_symbol(element.element_type, element.element_value)
    }

    fn add_nonterminal_with_name(&mut self, name: &str) -> SymbolRef<'a> {
        self.add_symbol(SymbolType::NonTerminal, name)
    }

    fn build_normalized_grammar<'input>(
        &mut self,
        symbols: &Vec<RulesNode<'input>>,
    ) -> Result<(), ParseError> {
        let mut used_sym = HashSet::new();
        for symbol in symbols.iter() {
            let mut used_ids = HashSet::new();
            let sym = symbol.sym;
            if used_sym.contains(sym) {
                return Err(ParseError::DuplicateSymbol(sym.to_string()));
            }
            used_sym.insert(sym);
            let left = self.add_nonterminal_with_name(sym);

            if symbol.root_symbol.is_some() {
                if self.start_symbol.is_some() {
                    return Err(ParseError::SecondRootSymbol(symbol.sym.into()));
                }
                self.start_symbol = Some(left);
            }

            for rule in symbol.alternatives.iter() {
                let rule_id = rule.id;
                let rule_len = rule.elements.len();
                if used_ids.contains(&rule_id) {
                    return Err(ParseError::DuplicateRuleId(sym.to_string()));
                }
                used_ids.insert(rule_id);
                let current_induction_name = format!("{sym}^{rule_id}");
                let current_induction = self.add_nonterminal_with_name(&current_induction_name);
                self.add_rule(
                    left,
                    Some(current_induction),
                    None,
                    GrammarRuleType::Induction,
                    left,
                    rule_id,
                    rule_len,
                    None,
                );

                match rule_len {
                    0 => {
                        self.add_rule(
                            current_induction,
                            None,
                            None,
                            GrammarRuleType::ConcatZero,
                            left,
                            rule_id,
                            rule_len,
                            None,
                        );
                    }
                    1 => {
                        let right1 = self.add_element(&rule.elements[0]);

                        self.add_rule(
                            current_induction,
                            Some(right1),
                            None,
                            GrammarRuleType::ConcatOne,
                            left,
                            rule_id,
                            rule_len,
                            None,
                        );
                    }
                    2 => {
                        let right1 = self.add_element(&rule.elements[0]);
                        let right2 = self.add_element(&rule.elements[1]);

                        self.add_rule(
                            current_induction,
                            Some(right1),
                            Some(right2),
                            GrammarRuleType::ConcatTwo,
                            left,
                            rule_id,
                            rule_len,
                            Some(1),
                        );
                    }
                    _ => {
                        let mut tmp_symbol = current_induction;
                        let mut tmp_symbol_id = 0;
                        for i in (2..rule_len).rev() {
                            let right1 = self.add_nonterminal_with_name(&format!(
                                "{current_induction_name}%{tmp_symbol_id}"
                            ));
                            tmp_symbol_id += 1;
                            let right2 = self.add_element(&rule.elements[i as usize]);
                            self.add_rule(
                                tmp_symbol,
                                Some(right1),
                                Some(right2),
                                GrammarRuleType::ConcatAppend,
                                left,
                                rule_id,
                                rule_len,
                                Some(i),
                            );
                            tmp_symbol = right1;
                        }

                        let right1 = self.add_element(&rule.elements[0]);
                        let right2 = self.add_element(&rule.elements[1]);
                        self.add_rule(
                            tmp_symbol,
                            Some(right1),
                            Some(right2),
                            GrammarRuleType::ConcatTwo,
                            left,
                            rule_id,
                            rule_len,
                            Some(1),
                        );
                    }
                }
            }
        }

        Ok(())
    }

    fn add_multi_valued_symbols_with_name(
        &mut self,
        multi_valued_symbols: &Vec<&str>,
    ) -> Result<(), ParseError> {
        let mut added_multi_value_symbols = HashSet::new();
        for symbol in multi_valued_symbols.iter() {
            if added_multi_value_symbols.contains(*symbol) {
                return Err(ParseError::DuplicateMultiValuedSymbol(symbol.to_string()));
            }
            added_multi_value_symbols.insert(*symbol);
            match self.find_symbol(SymbolType::SymbolicTerminal, *symbol) {
                Some(symbol) => {
                    self.add_multi_value_symbol(symbol);
                }
                None => {
                    return Err(ParseError::MultiValuedSymbolNotFound(symbol.to_string()));
                }
            }
        }
        Ok(())
    }

    fn build_grammar<'input>(&mut self, input: &'input str) -> Result<(), ParseError> {
        let parser = GrammarFileParser::new();
        let grammar_file = parser.parse(input)?;
        self.build_grammar_from_ast(&grammar_file)
    }

    fn add_unknown_symbol(&mut self) {
        let new_symbol_id = self.next_symbol_id;
        self.next_symbol_id += 1;
        let s = Symbol::new(new_symbol_id, SymbolType::LiteralTerminal, "__unknown__");
        let s = self.arena.symbols.alloc(s);
        self.unknown_terminal = Some(s);
    }

    fn build_grammar_from_ast<'input>(&mut self, ast: &GrammarFile<'_>) -> Result<(), ParseError> {
        self.add_unknown_symbol();
        self.build_normalized_grammar(&ast.rules)?;
        self.add_multi_valued_symbols_with_name(&ast.multivalued_symbols)?;
        self.check_grammar()?;

        Ok(())
    }

    fn check_start_symbol(&self) -> Result<(), ParseError> {
        if self.start_symbol.is_none() {
            Err(ParseError::StartSymbolNotFound())
        } else {
            Ok(())
        }
    }

    fn check_integrity(&self) -> Result<(), ParseError> {
        for rule in self.zero_productions.iter() {
            assert!(rule.right1().is_none());
            assert!(rule.right2().is_none());
        }
        for (_, symbol) in self
            .non_terminals
            .iter()
            .chain(self.literal_terimals.iter())
            .chain(self.symbolic_terminals.iter())
        {
            for ref_one_rule in symbol.ref_one(self).iter() {
                assert!(ref_one_rule.right1().is_some());
                assert!(ref_one_rule.right2().is_none());
                assert!(ref_one_rule.right1().unwrap() == *symbol);
            }
            for ref_two_rule in symbol.ref_two_left(self).iter() {
                assert!(ref_two_rule.right1().is_some());
                assert!(ref_two_rule.right2().is_some());
                assert!(ref_two_rule.right1().unwrap() == *symbol);
            }
            for ref_two_rule in symbol.ref_two_right(self).iter() {
                assert!(ref_two_rule.right1().is_some());
                assert!(ref_two_rule.right2().is_some());
                assert!(ref_two_rule.right2().unwrap() == *symbol);
            }
        }
        Ok(())
    }

    fn check_reachability(&self) -> Result<(), ParseError> {
        let mut reached_symbols: HashSet<SymbolRef<'a>> = HashSet::new();
        let mut queue: VecDeque<SymbolRef<'a>> = VecDeque::new();
        queue.push_back(self.start_symbol.unwrap());
        while !queue.is_empty() {
            let symbol = queue.pop_front().unwrap();
            if reached_symbols.contains(&symbol) {
                continue;
            }
            reached_symbols.insert(symbol);
            match symbol.symbol_type() {
                SymbolType::NonTerminal => {
                    if symbol.rules(self).len() == 0 {
                        return Err(ParseError::NonTerminalWithoutRules(
                            symbol.name().to_string(),
                        ));
                    }
                }
                _ => {}
            }
            for rule in symbol.rules(self).iter() {
                if rule.right1().is_some() {
                    queue.push_back(rule.right1().unwrap());
                }
                if rule.right2().is_some() {
                    queue.push_back(rule.right2().unwrap());
                }
            }
        }

        for (_, symbol) in self
            .non_terminals
            .iter()
            .chain(self.literal_terimals.iter())
            .chain(self.symbolic_terminals.iter())
        {
            if !reached_symbols.contains(symbol) {
                return Err(ParseError::UnReachableSymbol(symbol.name().to_string()));
            }
        }

        Ok(())
    }

    fn construct_epsilon_symbols(&self) -> HashSet<SymbolRef<'a>> {
        let mut epsilon_symbols_queue: VecDeque<SymbolRef<'a>> = VecDeque::new();
        for rule in self.zero_productions.iter() {
            epsilon_symbols_queue.push_back(rule.left());
        }
        let mut epsilon_symbols: HashSet<SymbolRef<'a>> = HashSet::new();
        while !epsilon_symbols_queue.is_empty() {
            let symbol = epsilon_symbols_queue.pop_front().unwrap();
            if epsilon_symbols.contains(&symbol) {
                continue;
            }
            epsilon_symbols.insert(symbol);
            for rule in symbol.ref_one(self).iter() {
                if !epsilon_symbols.contains(&rule.left()) {
                    epsilon_symbols_queue.push_back(rule.left());
                }
            }
            for rule in symbol.ref_two_left(self).iter() {
                if !epsilon_symbols.contains(&rule.left())
                    && epsilon_symbols.contains(&rule.right2().unwrap())
                {
                    epsilon_symbols_queue.push_back(rule.left());
                }
            }
            for rule in symbol.ref_two_right(self).iter() {
                if !epsilon_symbols.contains(&rule.left())
                    && epsilon_symbols.contains(&rule.right1().unwrap())
                {
                    epsilon_symbols_queue.push_back(rule.left());
                }
            }
        }
        epsilon_symbols
    }

    fn check_zero_loop(
        &self,
        symbol: SymbolRef<'a>,
        searching_symbols: &mut HashSet<SymbolRef<'a>>,
        reached_symbols: &mut HashSet<SymbolRef<'a>>,
        epsilon_symbols: &HashSet<SymbolRef<'a>>,
    ) -> Option<Vec<SymbolRef<'a>>> {
        if searching_symbols.contains(&symbol) {
            return Some(vec![symbol]);
        }
        if reached_symbols.contains(&symbol) {
            return None;
        }
        reached_symbols.insert(symbol);
        searching_symbols.insert(symbol);

        for rule in symbol.rules(self).iter() {
            match rule.rule_type().length() {
                GrammarRuleLength::Zero => {}
                GrammarRuleLength::One => {
                    let mut result = self.check_zero_loop(
                        rule.right1().unwrap(),
                        searching_symbols,
                        reached_symbols,
                        epsilon_symbols,
                    );
                    match result {
                        Some(ref mut vec) => {
                            vec.push(symbol);
                            return Some(vec.clone());
                        }
                        None => {}
                    }
                }
                GrammarRuleLength::Two => {
                    if epsilon_symbols.contains(&rule.right1().unwrap()) {
                        let mut result = self.check_zero_loop(
                            rule.right2().unwrap(),
                            searching_symbols,
                            reached_symbols,
                            epsilon_symbols,
                        );
                        match result {
                            Some(ref mut vec) => {
                                vec.push(symbol);
                                return Some(vec.clone());
                            }
                            None => {}
                        }
                    }
                    if epsilon_symbols.contains(&rule.right2().unwrap()) {
                        let mut result = self.check_zero_loop(
                            rule.right1().unwrap(),
                            searching_symbols,
                            reached_symbols,
                            epsilon_symbols,
                        );
                        match result {
                            Some(ref mut vec) => {
                                vec.push(symbol);
                                return Some(vec.clone());
                            }
                            None => {}
                        }
                    }
                }
            }
        }

        searching_symbols.remove(&symbol);
        None
    }

    fn check_zero_loops(&self) -> Result<(), ParseError> {
        let epsilon_symbols = self.construct_epsilon_symbols();
        for (_, symbol) in self.non_terminals.iter() {
            let mut searching_symbols: HashSet<SymbolRef<'a>> = HashSet::new();
            let mut reached_symbols: HashSet<SymbolRef<'a>> = HashSet::new();
            if let Some(cycle) = self.check_zero_loop(
                *symbol,
                &mut searching_symbols,
                &mut reached_symbols,
                &epsilon_symbols,
            ) {
                let mut result_string = "[".to_string();
                for symbol_in_loop in cycle.iter().rev() {
                    write!(&mut result_string, "{} ", symbol_in_loop.name()).unwrap();
                }
                write!(&mut result_string, "]").unwrap();
                return Err(ParseError::ZeroLoop(result_string));
            }
        }

        Ok(())
    }

    fn check_grammar(&self) -> Result<(), ParseError> {
        self.check_start_symbol()?;
        self.check_integrity()?;
        self.check_reachability()?;
        self.check_zero_loops()?;

        Ok(())
    }

    pub fn new<'input>(
        arena: &'a GrammarArena<'a>,
        input: &'input str,
    ) -> Result<Self, ParseError> {
        let mut grammar = Grammar::construct(arena);
        grammar.build_grammar(input)?;
        Ok(grammar)
    }

    pub fn from_grammar_ast(
        arena: &'a GrammarArena<'a>,
        ast: &GrammarFile<'_>,
    ) -> Result<Self, ParseError> {
        let mut grammar = Grammar::construct(arena);
        grammar.build_grammar_from_ast(ast)?;
        Ok(grammar)
    }

    pub fn get_symbol_ref(&'a self) -> GrammarSymbolsRef<'a> {
        GrammarSymbolsRef {
            literal_terminals: &self.literal_terimals,
            symbolic_terminals: &self.symbolic_terminals,
            non_terminals: &self.non_terminals,
            zero_productions: &self.zero_productions,
            start_symbol: self.start_symbol.unwrap(),
            unknown_terminal: self.unknown_terminal.unwrap(),
        }
    }
}

impl<'a> Display for Grammar<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Non-terminals:")?;
        for (_, symbol) in self.non_terminals.iter() {
            write!(f, " ")?;
            symbol.ptr().fmt_all(self, f)?;
        }
        writeln!(f, "Literal-terminals:")?;
        for (_, symbol) in self.literal_terimals.iter() {
            write!(f, " ")?;
            symbol.ptr().fmt_all(self, f)?;
        }
        writeln!(f, "Symbolic-terminals:")?;
        for (_, symbol) in self.symbolic_terminals.iter() {
            write!(f, " ")?;
            symbol.ptr().fmt_all(self, f)?;
        }
        Ok(())
    }
}

pub struct GrammarArena<'a> {
    symbols: RefArena<Symbol<'a>>,
    rules: RefArena<GrammarRule<'a>>,
}

impl<'a> GrammarArena<'a> {
    pub fn new() -> Self {
        Self {
            symbols: RefArena::new(),
            rules: RefArena::new(),
        }
    }
}
