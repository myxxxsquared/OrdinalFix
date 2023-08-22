use fixing_rs_base::{
    grammar::{GrammarSymbolsRef, OwnedToken, SymbolType},
    tokenizer::{Token, Tokenizer},
};
use mj_token_parser::FileParser;
use std::{
    error::Error,
    fmt::{Debug, Display, Formatter},
};

lalrpop_mod!(mj_token_parser, "/grammars/mj_tokenizer.rs");

#[derive(Debug)]
pub enum MJToken<'input> {
    LiteralTerminal(&'input str),
    Identifier(&'input str),
}

pub struct MJTokenizer;

impl Tokenizer for MJTokenizer {
    type ErrType = MJTokenizerError;
    fn tokenize<'a, 's>(
        &mut self,
        input: &'s str,
        grammar: GrammarSymbolsRef<'a>,
    ) -> Result<Vec<Token<'a, 's>>, Self::ErrType> {
        let mut result = Vec::new();
        let parser = FileParser::new();
        let tokens = parser.parse(input)?;
        let literal_terminals = grammar.literal_terminals;
        let symbolic_terminals = grammar.symbolic_terminals;
        let identifier_terminal =
            *(symbolic_terminals
                .get("IDENTIFIER")
                .ok_or(MJTokenizerError::SymbolNotFound(
                    SymbolType::SymbolicTerminal,
                    "IDENTIFIER".to_string(),
                ))?);
        for token in tokens {
            match token {
                MJToken::LiteralTerminal(literal) => {
                    let symbol =
                        literal_terminals
                            .get(literal)
                            .ok_or(MJTokenizerError::SymbolNotFound(
                                SymbolType::LiteralTerminal,
                                literal.to_string(),
                            ))?;
                    result.push(Token {
                        literal,
                        symbol: *symbol,
                    })
                }
                MJToken::Identifier(identifier) => result.push(Token {
                    literal: identifier,
                    symbol: identifier_terminal,
                }),
            }
        }
        Ok(result)
    }
}

#[derive(Debug)]
pub enum MJTokenizerError {
    SyntaxError(lalrpop_util::ParseError<usize, OwnedToken, &'static str>),
    SymbolNotFound(SymbolType, String),
}

impl Display for MJTokenizerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

impl Error for MJTokenizerError {}

impl<'input> From<lalrpop_util::ParseError<usize, lalrpop_util::lexer::Token<'input>, &'static str>>
    for MJTokenizerError
{
    fn from(
        input: lalrpop_util::ParseError<usize, lalrpop_util::lexer::Token<'input>, &'static str>,
    ) -> Self {
        Self::SyntaxError(input.map_token(|x| x.into()))
    }
}
