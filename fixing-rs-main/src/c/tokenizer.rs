use fixing_rs_base::{
    grammar::GrammarSymbolsRef,
    tokenizer::{Token, Tokenizer},
};
use std::{
    error::Error,
    fmt::{Debug, Display},
};

pub struct CTokenizer;

#[derive(Debug)]
pub enum CParseError {
    LineFormatError(String, usize),
    UnknownTy(String, usize),
    UnknownName(String, usize),
}

impl Display for CParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

impl Error for CParseError {}

impl Tokenizer for CTokenizer {
    type ErrType = CParseError;

    fn tokenize<'a, 's>(
        &mut self,
        input: &'s str,
        grammar: GrammarSymbolsRef<'a>,
    ) -> Result<Vec<fixing_rs_base::tokenizer::Token<'a, 's>>, Self::ErrType> {
        let mut result = Vec::new();
        for (line_id, line) in input.split("\n").enumerate() {
            let line = line.trim();
            if line.len() == 0 {
                continue;
            }
            let line_split: [&str; 3] = line
                .split("\t")
                .take(4)
                .collect::<Vec<_>>()
                .try_into()
                .map_err(|_| CParseError::LineFormatError(line.to_string(), line_id + 1))?;
            let [ty, name, literal] = line_split;
            let m = match ty {
                "LT" => &grammar.literal_terminals,
                "ST" => &grammar.symbolic_terminals,
                _ => return Err(CParseError::UnknownTy(ty.to_string(), line_id + 1)),
            };
            match m.get(name) {
                Some(symbol) => result.push(Token {
                    symbol: *symbol,
                    literal,
                }),
                _ => {
                    result.push(Token { literal: literal, symbol: grammar.unknown_terminal })
                }
            }
        }
        Ok(result)
    }
}
