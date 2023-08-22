use super::{super::GReachabilityArena, Edge, EdgeMap, GKey, GKeyRef, GProcessor, GRule, GRuleRef};
use crate::{
    containers::{Map, Set},
    grammar::{Grammar, GrammarRuleRef, GrammarRuleType, GrammarSymbolsRef, SymbolRef, SymbolType},
    props::{IntoPropResult, PropArray, PropResult, UnionProp},
    tokenizer::Token,
    utils::Queue,
};
use std::fmt::Display;

struct SymbolQuickRef<'a, 'b, PG>
where
    PG: UnionProp,
{
    ref_right: Map<(SymbolRef<'a>, usize, usize), Set<GKeyRef<'a, 'b, PG>>>,
    ref_left: Map<(SymbolRef<'a>, usize, usize), Set<GKeyRef<'a, 'b, PG>>>,
}

impl<'a, 'b, PG> SymbolQuickRef<'a, 'b, PG>
where
    PG: UnionProp,
{
    fn new() -> Self {
        Self {
            ref_right: Map::new(),
            ref_left: Map::new(),
        }
    }
    fn get_ref<const RIGHT: bool>(
        &self,
    ) -> &Map<(SymbolRef<'a>, usize, usize), Set<GKeyRef<'a, 'b, PG>>> {
        if RIGHT {
            &self.ref_right
        } else {
            &self.ref_left
        }
    }
    fn get_ref_mut<const RIGHT: bool>(
        &mut self,
    ) -> &mut Map<(SymbolRef<'a>, usize, usize), Set<GKeyRef<'a, 'b, PG>>> {
        if RIGHT {
            &mut self.ref_right
        } else {
            &mut self.ref_left
        }
    }
    fn add<const RIGHT: bool>(&mut self, edge: GKeyRef<'a, 'b, PG>) {
        let key = (
            edge.symbol(),
            if RIGHT { edge.begin() } else { edge.end() },
            edge.length(),
        );
        let r = self.get_ref_mut::<RIGHT>();
        if !r.contains_key(&key) {
            r.insert(key, Set::new());
        }
        r.get_mut(&key).unwrap().insert(edge);
    }
}

pub struct GReachability<'a, 'b, 'p, PG, GProc>
where
    PG: UnionProp,
    GProc: GProcessor<PG = PG>,
{
    grammar_ref: GrammarSymbolsRef<'a>,
    grammar: &'a Grammar<'a>,
    arena: &'b GReachabilityArena<'a, 'b, PG>,
    edges: EdgeMap<'a, 'b, GKey<'a, PG>, Map<&'b GRule<'a, 'b, PG>, GRuleRef<'a, 'b, PG>>>,
    literals: Vec<&'b str>,
    to_update: Queue<GKeyRef<'a, 'b, PG>>,
    max_length: usize,
    token_length: usize,
    processor: &'p GProc,
    quick_ref: SymbolQuickRef<'a, 'b, PG>,
    start_edge: Vec<Vec<GKeyRef<'a, 'b, PG>>>,
    next_updated_length: usize,
}

impl<'a, 'b, 'p, PG, GProc> GReachability<'a, 'b, 'p, PG, GProc>
where
    PG: UnionProp,
    GProc: GProcessor<PG = PG>,
{
    pub fn new(
        grammar: &'a Grammar<'a>,
        arena: &'b GReachabilityArena<'a, 'b, PG>,
        tokens: &Vec<Token<'a, '_>>,
        processor: &'p GProc,
        max_length: usize,
    ) -> Self {
        let mut result = Self {
            grammar,
            grammar_ref: grammar.get_symbol_ref(),
            arena,
            edges: EdgeMap::new(tokens.len(), max_length),
            literals: Vec::new(),
            max_length,
            to_update: Queue::new(max_length),
            token_length: tokens.len(),
            processor,
            quick_ref: SymbolQuickRef::new(),
            start_edge: Vec::new(),
            next_updated_length: 0,
        };
        result.add_originals(tokens, processor);
        if max_length > 0 {
            result.add_modifications();
        }
        result.update0();
        result
    }

    fn add_originals(&mut self, tokens: &Vec<Token<'a, '_>>, processor: &impl GProcessor<PG = PG>) {
        for i in 0..tokens.len() {
            let token = &tokens[i];
            let literal = self.arena.tokens.alloc(token.literal.to_owned());
            let literal = &literal.ptr()[..];
            self.literals.push(literal);
            let symbol = token.symbol;
            let prop = match symbol.symbol_type() {
                SymbolType::LiteralTerminal => PG::default().into_prop_result(),
                SymbolType::SymbolicTerminal => {
                    processor.process_symbolic_terminal(symbol, Some(literal))
                }
                _ => panic!(
                    "Any token should not be non-terminal: {:?}",
                    token.literal.to_owned()
                ),
            };
            prop.consume(|p| {
                self.add_edge(i, i + 1, symbol, 0, PropArray::Single(p));
            });
        }
    }

    fn add_modifications(&mut self) {
        let GrammarSymbolsRef {
            literal_terminals,
            symbolic_terminals,
            ..
        } = self.grammar_ref;
        let mod_edges = (0..self.token_length).map(|x| (x, x + 1));
        let insert_edges = (0..self.token_length + 1).map(|x| (x, x));
        for (loc_begin, loc_end) in mod_edges.chain(insert_edges) {
            for (_, symbol) in literal_terminals.iter() {
                self.add_edge(
                    loc_begin,
                    loc_end,
                    *symbol,
                    1,
                    PropArray::Single(PG::default()),
                );
            }
            for (_, symbol) in symbolic_terminals.iter() {
                let prop = self.processor.process_symbolic_terminal(*symbol, None);
                prop.consume(|p| {
                    self.add_edge(loc_begin, loc_end, *symbol, 1, PropArray::Single(p));
                });
            }
        }
    }

    fn add_edge(
        &mut self,
        begin: usize,
        end: usize,
        symbol: SymbolRef<'a>,
        length: usize,
        prop: PropArray<PG>,
    ) -> GKeyRef<'a, 'b, PG> {
        let edge_key = GKey::new(begin, end, symbol, length, prop);
        if let Some((key, _)) = self.edges.get(&edge_key) {
            return key;
        }
        if symbol.name() == "argumentList^0" {
            if let PropArray::Single(_) = edge_key.prop() {
                panic!("add_edge failed.");
            }
        }
        let key = self.arena.gedges.alloc(edge_key);
        self.edges.insert_default(key);
        self.to_update.push(key, key.length());
        self.quick_ref.add::<true>(key);
        self.quick_ref.add::<false>(key);
        self.try_put_into_start_edge(key);
        key
    }

    fn try_put_into_start_edge(&mut self, key: GKeyRef<'a, 'b, PG>) {
        let GrammarSymbolsRef { start_symbol, .. } = self.grammar_ref;
        if key.symbol() != start_symbol {
            return;
        }
        let length = key.length();
        while self.start_edge.len() <= length {
            self.start_edge.push(Vec::new());
        }
        self.start_edge.get_mut(length).unwrap().push(key);
    }

    fn add_generation(
        &mut self,
        edge: GKeyRef<'a, 'b, PG>,
        sub1: Option<GKeyRef<'a, 'b, PG>>,
        sub2: Option<GKeyRef<'a, 'b, PG>>,
        rule: GrammarRuleRef<'a>,
    ) {
        if edge.length() < self.next_updated_length {
            panic!("Edge wrong generated length.")
        }
        let (_, generations) = self.edges.get_mut(&*edge).unwrap();
        let entity = GRule::new(sub1, sub2, rule);
        let entity = self.arena.grules.alloc(entity);
        generations.insert(entity.ptr(), entity);
    }

    fn update0(&mut self) {
        let GrammarSymbolsRef {
            zero_productions, ..
        } = self.grammar_ref;

        for rule in zero_productions {
            let symbol = rule.left();
            let prop = self.process_zero(*rule);
            for i in 0..=self.token_length {
                prop.clone().consume(|p| {
                    let edge = self.add_edge(i, i, symbol, 0, p);
                    self.add_generation(edge, None, None, *rule);
                });
            }
        }
    }
    fn update1(&mut self, edge: GKeyRef<'a, 'b, PG>) {
        let GrammarSymbolsRef { start_symbol, .. } = self.grammar_ref;
        let symbol = edge.symbol();
        let ref_one = symbol.ref_one(&self.grammar);
        for rule in ref_one.iter() {
            if rule.left() == start_symbol {
                let total_len = edge.length()
                    + edge.begin()
                    + (self.token_length.checked_sub(edge.end()).unwrap());
                if total_len <= self.max_length {
                    let prop = self.processor_one(*rule, &edge.prop());
                    prop.consume(|p| {
                        let genedge =
                            self.add_edge(0, self.token_length, start_symbol, total_len, p);
                        self.add_generation(genedge, Some(edge), None, *rule);
                    });
                }
            } else {
                let prop = self.processor_one(*rule, edge.prop());
                prop.consume(|p| {
                    let genedge =
                        self.add_edge(edge.begin(), edge.end(), rule.left(), edge.length(), p);
                    self.add_generation(genedge, Some(edge), None, *rule);
                });
            }
        }
    }
    fn update2<const RIGHT: bool>(
        &mut self,
        edge: GKeyRef<'a, 'b, PG>,
        from_length: usize,
        to_length: usize,
    ) {
        let symbol = edge.symbol();
        for rule in {
            if RIGHT {
                symbol.ref_two_left(&self.grammar)
            } else {
                symbol.ref_two_right(&self.grammar)
            }
        }
        .iter()
        {
            let loc_max = if RIGHT {
                self.token_length.checked_sub(edge.end()).unwrap()
            } else {
                edge.begin()
            };
            let other_symbol = if RIGHT { rule.right2() } else { rule.right1() };
            let other_symbol = other_symbol.unwrap();
            let loc_max = std::cmp::min(loc_max, to_length.checked_sub(edge.length()).unwrap());
            for i in 0..=loc_max {
                let cur_loc = if RIGHT {
                    let x = edge.end() + i;
                    if x > self.token_length {
                        break;
                    }
                    x
                } else {
                    match edge.begin().checked_sub(i) {
                        Some(x) => x,
                        None => break,
                    }
                };
                let length_from = if from_length >= edge.length() + i {
                    from_length.checked_sub(edge.length() + i).unwrap()
                } else {
                    0
                };
                let length_to = to_length.checked_sub(edge.length() + i).unwrap();
                for l in length_from..=length_to {
                    let key = (other_symbol, cur_loc, l);
                    if let Some(edges) = self.quick_ref.get_ref::<RIGHT>().get(&key) {
                        for other_edge in edges.clone() {
                            let right1 = if RIGHT { &edge } else { &other_edge };
                            let right2 = if RIGHT { &other_edge } else { &edge };
                            let prop = self.process_two(*rule, right1.prop(), right2.prop());
                            prop.consume(|p| {
                                let genedge = self.add_edge(
                                    right1.begin(),
                                    right2.end(),
                                    rule.left(),
                                    i + right1.length() + right2.length(),
                                    p,
                                );
                                self.add_generation(genedge, Some(*right1), Some(*right2), *rule);
                            });
                        }
                    }
                }
            }
        }
    }

    fn process_zero(&self, rule: GrammarRuleRef<'a>) -> PropResult<PropArray<PG>> {
        match rule.rule_type() {
            GrammarRuleType::ConcatZero => PropArray::Multiple(vec![].into()).into_prop_result(),
            _ => panic!("Invoking processor_zero with rule {:?}", rule),
        }
    }
    fn processor_one(
        &self,
        rule: GrammarRuleRef<'a>,
        p: &PropArray<PG>,
    ) -> PropResult<PropArray<PG>> {
        match rule.rule_type() {
            GrammarRuleType::Induction => self
                .processor
                .process_non_terminal(rule.left(), rule.induction_id(), p.unwrap_multiple())
                .into(),
            GrammarRuleType::ConcatOne => {
                PropArray::Multiple(vec![p.unwrap_single().clone()].into()).into_prop_result()
            }
            _ => panic!("Invoking processor_one with rule {:?}", rule),
        }
    }
    fn process_two(
        &self,
        rule: GrammarRuleRef<'a>,
        p1: &PropArray<PG>,
        p2: &PropArray<PG>,
    ) -> PropResult<PropArray<PG>> {
        match rule.rule_type() {
            GrammarRuleType::ConcatAppend => {
                p1.append(p2.unwrap_single().clone()).into_prop_result()
            }
            GrammarRuleType::ConcatTwo => PropArray::Multiple(
                vec![p1.unwrap_single().clone(), p2.unwrap_single().clone()].into(),
            )
            .into_prop_result(),
            _ => panic!("Invoking processor_one with rule {:?}", rule),
        }
    }
}

impl<'a, 'b, 'p, PG, GProc> GReachability<'a, 'b, 'p, PG, GProc>
where
    PG: UnionProp,
    GProc: GProcessor<PG = PG>,
{
    pub fn get_start_edges(&self) -> &Vec<Vec<GKeyRef<'a, 'b, PG>>> {
        &self.start_edge
    }

    pub fn get_sub_edges(
        &self,
        edge: GKeyRef<'a, 'b, PG>,
    ) -> &Map<&'b GRule<'a, 'b, PG>, GRuleRef<'a, 'b, PG>> {
        &self.edges.get(edge.ptr()).unwrap().1
    }
    pub fn update_until(&mut self, max_length: usize) {
        let max_length = std::cmp::min(max_length, self.max_length);
        if max_length < self.next_updated_length {
            return;
        }
        for current_length in 0..self.next_updated_length {
            let mut idx = self.to_update.index_from_begin(current_length);
            while let Some(edge) = self.to_update.get_next(&mut idx, current_length) {
                self.update2::<true>(edge, self.next_updated_length, max_length);
                self.update2::<false>(edge, self.next_updated_length, max_length);
            }
        }
        for current_length in self.next_updated_length..=max_length {
            while let Some(edge) = self.to_update.queue_next(current_length) {
                self.update1(edge);
                self.update2::<true>(edge, self.next_updated_length, max_length);
                self.update2::<false>(edge, self.next_updated_length, max_length);
            }
        }
        self.next_updated_length = max_length + 1;
    }
    pub fn literals(&self) -> &Vec<&'b str> {
        &self.literals
    }
}

impl<'a, 'b, 'p, PG, GProc> Display for GReachability<'a, 'b, 'p, PG, GProc>
where
    PG: UnionProp,
    GProc: GProcessor<PG = PG>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut edges = self.edges.all_edges();
        edges.sort_by_key(|e| {
            let edge = e;
            (
                edge.length(),
                edge.begin(),
                edge.end(),
                String::from(edge.symbol().name()),
            )
        });
        for edge in edges {
            let rules = self.edges.get(edge.ptr()).unwrap().1;
            writeln!(f, "Edge: {}", edge.ptr())?;
            for rule_ref in rules.values() {
                writeln!(f, " {}", rule_ref.ptr())?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
