use crate::mj::{fixing::MJ_GRAMMAR, syntactic::MJGProcessor, tokenizer::MJTokenizer};
use fixing_rs_base::{
    grammar::{Grammar, GrammarArena},
    reachability::{GReachability, GReachabilityArena},
    tokenizer::Tokenizer,
};
use std::{fs::File, io::Write};

const TEST_SYNTACTIC: &'static [(&'static str, usize)] = &[
    (
        "{if (a == b) { a = b; } else { c = (D)(new MYCLS(x, y, z)); } x.y = a; return null;}",
        0,
    ),
    ("", 2),
    (
        "{if (a b) { a = b; } else { c = (D)(new MYCLS(x, y, z)); } x.y = a; return null;}",
        1,
    ),
];

#[test]
fn test_syntactic() {
    let grammar_arena = GrammarArena::new();
    let grammar = Grammar::new(&grammar_arena, MJ_GRAMMAR).unwrap();
    let symbol_ref = grammar.get_symbol_ref();
    for (t_str, t_len) in TEST_SYNTACTIC {
        let syntactic_arena = GReachabilityArena::new();
        let tokens = MJTokenizer.tokenize(t_str, symbol_ref).unwrap();
        let mut syntactic_reachability =
            GReachability::new(&grammar, &syntactic_arena, &tokens, &MJGProcessor, *t_len);
        syntactic_reachability.update_until(*t_len);
        let mut output_file = File::create("../target/mjgrammar-syntactic-reachability").unwrap();
        write!(&mut output_file, "{}", syntactic_reachability).unwrap();
        syntactic_reachability
            .get_start_edges()
            .get(*t_len)
            .unwrap()
            .first()
            .unwrap();
    }
}
