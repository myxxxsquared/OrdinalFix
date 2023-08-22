use super::{cenv::CEnvBuildError, tokenizer::CParseError};
use crate::c::{
    cenv::{CEnv, CEnvArena},
    semantic::CSProcessor,
    syntactic::CGProcessor,
    tokenizer::CTokenizer,
    types::{CTypeArena, CTypePool},
};
use fixing_rs_base::{
    fixing::{
        DoFix, FixTaskError, FixTaskInfo, FixTaskResult, FixingInputProcessor,
        FixingInputProcessorBase,
    },
    fixing_info::FixingInfo,
    grammar::Grammar,
    tokenizer::Tokenizer,
};
use std::time::Instant;

pub struct CFixingInputProcessor;

impl FixingInputProcessorBase for CFixingInputProcessor {
    fn info(&self) -> &FixingInfo {
        &C_FIXING_INFO
    }
}

impl FixingInputProcessor for CFixingInputProcessor {
    fn process<'a>(
        &self,
        grammar: &'a Grammar<'a>,
        input_str: &str,
        env_str: &str,
        info: &FixTaskInfo,
        time_before_load: Instant,
        do_fix: impl DoFix,
    ) -> Result<FixTaskResult, FixTaskError<Self::TokenizerError, Self::EnvLoadError>> {
        let symbol_ref = grammar.get_symbol_ref();
        let tokens = CTokenizer
            .tokenize(input_str, symbol_ref)
            .map_err(|e| FixTaskError::TokenizerError(e))?;
        let arena = CEnvArena::new();
        let types_arena = CTypeArena::new();
        let types = CTypePool::new(&types_arena);
        let env = CEnv::build(&arena, env_str, &types, &tokens, info.max_new_id)
            .map_err(|e| FixTaskError::EnvLoadError(e))?;
        let sproc = CSProcessor::new(&env);
        let gproc = CGProcessor;

        do_fix.do_fix(grammar, &tokens, &gproc, &sproc, info, time_before_load)
    }

    type TokenizerError = CParseError;
    type EnvLoadError = CEnvBuildError;
}

pub const C_GRAMMAR: &'static str = include_str!("c_grammar");
pub const C_GRAMMAR_FILE: &'static str = "src/c/c_grammar";
pub const C_PROP_G: &'static str = "CProp";
pub const C_PROP_SI: &'static str = "CInhProp";
pub const C_PROP_SS: &'static str = "CSynProp";
pub const C_ENTITY_I: &'static str = "CInhEntity<'s>";
pub const C_CONTAINER_I: &'static str = "CInhEntityArena";
pub const C_ENTITY_S: &'static str = "CSynEntity<'s>";
pub const C_CONTAINER_S: &'static str = "CSynEntityArena";

pub const C_FIXING_INFO: FixingInfo = FixingInfo {
    grammar: C_GRAMMAR,
    grammar_file: C_GRAMMAR_FILE,
    prop_g: C_PROP_G,
    prop_si: C_PROP_SI,
    prop_ss: C_PROP_SS,
    entity_i: C_ENTITY_I,
    entity_s: C_ENTITY_S,
    container_i: C_CONTAINER_I,
    container_s: C_CONTAINER_S,
};
