use crate::grammar::{GrammarSymbolsRef, SymbolRef};
use std::error::Error;

#[derive(Debug)]
pub struct Token<'a, 's> {
    pub literal: &'s str,
    pub symbol: SymbolRef<'a>,
}

pub trait Tokenizer {
    type ErrType: Error;

    fn tokenize<'a, 's>(
        &mut self,
        input: &'s str,
        grammar: GrammarSymbolsRef<'a>,
    ) -> Result<Vec<Token<'a, 's>>, Self::ErrType>;
}
