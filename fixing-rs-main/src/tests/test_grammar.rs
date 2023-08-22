use crate::mj::fixing::MJ_GRAMMAR;
use fixing_rs_base::grammar::{Grammar, GrammarArena, ParseError};
use std::{fs, io::Write};

#[test]
fn test_load_grammar() {
    let arena = GrammarArena::new();
    let grammar = MJ_GRAMMAR;
    let grammar_loaded = Grammar::new(&arena, &grammar).unwrap();
    let mut output_file = fs::File::create("../target/mjgrammar-dump").unwrap();
    write!(&mut output_file, "{}", grammar_loaded).unwrap();
}

#[test]
fn test_unreachable() {
    let arena = GrammarArena::new();
    let grammar = include_str!("test_grammar/test_grammar_unreachable");
    let err = Grammar::new(&arena, &grammar).map(|_| ()).unwrap_err();
    match err {
        ParseError::UnReachableSymbol(_) => {}
        _ => panic!("Unexpected error: {:?}", err),
    }
}

#[test]
fn test_grammar_zero_loop() {
    let arena = GrammarArena::new();
    let grammar = include_str!("test_grammar/test_grammar_loop");
    let err = Grammar::new(&arena, &grammar).map(|_| ()).unwrap_err();
    match err {
        ParseError::ZeroLoop(s) => {
            println!("Zero loop: {}", s)
        }
        _ => panic!("Unexpected error: {:?}", err),
    }
}
