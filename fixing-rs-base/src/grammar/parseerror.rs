use lalrpop_util::lexer::Token;
use std::{
    error::Error,
    fmt::{Display, Formatter},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseError {
    SyntaxError(lalrpop_util::ParseError<usize, OwnedToken, &'static str>),
    DuplicateSymbol(String),
    DuplicateRuleId(String),
    StartSymbolNotFound(),
    MultiValuedSymbolNotFound(String),
    DuplicateMultiValuedSymbol(String),
    UnReachableSymbol(String),
    NonTerminalWithoutRules(String),
    ZeroLoop(String),
    SecondRootSymbol(String),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ParseError {}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct OwnedToken(pub usize, pub String);

impl<'input> From<Token<'input>> for OwnedToken {
    fn from(token: Token<'input>) -> Self {
        let Token(size, string) = token;
        Self(size, string.to_string())
    }
}

impl<'input> From<lalrpop_util::ParseError<usize, Token<'input>, &'static str>> for ParseError {
    fn from(input: lalrpop_util::ParseError<usize, Token<'input>, &'static str>) -> Self {
        Self::SyntaxError(input.map_token(|x| x.into()))
    }
}
