use super::fixing::C_GRAMMAR;
use fixing_rs_base::grammar::{Grammar, GrammarArena};

#[test]
fn test_load_c_grammar() {
    let arena = GrammarArena::new();
    let grammar = Grammar::new(&arena, C_GRAMMAR).unwrap();
    println!("{}", grammar);
}
