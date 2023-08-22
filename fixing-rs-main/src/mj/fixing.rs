use super::{
    mjenv::{MJArena, MJEnv, MJParseError},
    semantic::MJSProcessor,
    syntactic::MJGProcessor,
    tokenizer::{MJTokenizer, MJTokenizerError},
};
use fixing_rs_base::{
    fixing::{
        FixTaskError, FixTaskInfo, FixTaskResult, FixingInputProcessor, FixingInputProcessorBase,
    },
    fixing_info::FixingInfo,
    grammar::Grammar,
    tokenizer::Tokenizer,
    utils::RefArena,
};
use std::time::Instant;

pub struct MJFixingInputProcessor;

impl FixingInputProcessorBase for MJFixingInputProcessor {
    fn info(&self) -> &'static FixingInfo {
        &MJ_FIXING_INFO
    }
}

impl FixingInputProcessor for MJFixingInputProcessor {
    fn process<'a>(
        &self,
        grammar: &'a Grammar<'a>,
        input_str: &str,
        env_str: &str,
        info: &FixTaskInfo,
        time_before_load: Instant,
        do_fix: impl fixing_rs_base::fixing::DoFix,
    ) -> Result<FixTaskResult, FixTaskError<Self::TokenizerError, Self::EnvLoadError>> {
        let symbol_ref = grammar.get_symbol_ref();
        let mjarena = MJArena::new();
        let env =
            MJEnv::build_from_env(&mjarena, env_str).map_err(|e| FixTaskError::EnvLoadError(e))?;
        let tokens = MJTokenizer
            .tokenize(input_str, symbol_ref)
            .map_err(|e| FixTaskError::TokenizerError(e))?;
        let strs = RefArena::new();
        let gproc = MJGProcessor;
        let sproc = MJSProcessor::new(&env, &strs, &tokens, info.max_new_id);

        do_fix.do_fix(grammar, &tokens, &gproc, &sproc, info, time_before_load)
    }

    type TokenizerError = MJTokenizerError;
    type EnvLoadError = MJParseError;
}

pub const MJ_GRAMMAR: &str = include_str!("middle_weight_java");
pub const MJ_GRAMMAR_FILE: &str = "src/mj/middle_weight_java";
pub const MJ_PROP_G: &str = "MJProp";
pub const MJ_PROP_SI: &str = "MJInhProp";
pub const MJ_PROP_SS: &str = "MJSynProp";
pub const MJ_ENTITY_I: &str = "MJInhEntity<'s>";
pub const MJ_CONTAINER_I: &str = "MJInhEntityArena";
pub const MJ_ENTITY_S: &str = "MJSynEntity<'s>";
pub const MJ_CONTAINER_S: &str = "MJSynEntityArena";

pub const MJ_FIXING_INFO: FixingInfo = FixingInfo {
    grammar: MJ_GRAMMAR,
    grammar_file: MJ_GRAMMAR_FILE,
    prop_g: MJ_PROP_G,
    prop_si: MJ_PROP_SI,
    prop_ss: MJ_PROP_SS,
    entity_i: MJ_ENTITY_I,
    entity_s: MJ_ENTITY_S,
    container_i: MJ_CONTAINER_I,
    container_s: MJ_CONTAINER_S,
};
