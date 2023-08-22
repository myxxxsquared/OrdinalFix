use fixing_rs_base::{
    grammar::{Grammar, GrammarArena},
    tokenizer::Tokenizer,
};

use crate::mj::{fixing::MJ_GRAMMAR, tokenizer::MJTokenizer};

const TOKENIZER_INPUT: &'static str =
    "if (a == b) { a = b; } else { c = new MYCLS(x, y, z); } x.y = 10; return null;";

#[test]
fn test_tokenizer() {
    let grammar_arena = GrammarArena::new();
    let grammar = Grammar::new(&grammar_arena, MJ_GRAMMAR).unwrap();
    let tokens = MJTokenizer
        .tokenize(TOKENIZER_INPUT, grammar.get_symbol_ref())
        .unwrap();
    for token in tokens {
        println!("{:?}", token);
    }
}
