use crate::{
    fixing_info::FixingInfo,
    grammar::{Grammar, GrammarArena},
    props::UnionProp,
    reachability::{
        find, GProcessor, GReachability, GReachabilityArena, SProcessor, SReachability,
        SReachabilityArena,
    },
    tokenizer::Token,
};
use log::info;
use std::{
    error::Error,
    fmt::{Debug, Display},
    fs,
    io::{self, Write},
    time::{Duration, Instant},
};

pub struct FixTaskInfo {
    pub input_name: String,
    pub env_name: String,
    pub output_name: Option<String>,
    pub max_len: usize,
    pub max_new_id: usize,
    pub verbose_gen: bool,
}

#[derive(Debug)]
pub struct FixTaskResult {
    pub time_before_load: Instant,
    pub time_after_load: Instant,
    pub time_after_reachability_built: Instant,
    pub time_after_find: Vec<Instant>,
    pub found_length: Option<usize>,
    pub outputs: Option<Vec<String>>,
}

#[derive(Debug)]
pub enum FixTaskError<T: Error, E: Error> {
    ReadInputError(io::Error),
    ReadEnvError(io::Error),
    WriteOutputError(io::Error),
    TokenizerError(T),
    EnvLoadError(E),
}

impl<T: Debug + Error, E: Debug + Error> Display for FixTaskError<T, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

impl<T: Error, E: Error> Error for FixTaskError<T, E> {}

fn do_fix_impl<'a, GProc, SProc, PG, T, E>(
    grammar: &'a Grammar<'a>,
    tokens: &Vec<Token<'a, '_>>,
    gproc: &GProc,
    sproc: &SProc,
    info: &FixTaskInfo,
    time_before_load: Instant,
) -> Result<FixTaskResult, FixTaskError<T, E>>
where
    PG: UnionProp,
    GProc: GProcessor<PG = PG>,
    SProc: SProcessor<PG = PG>,
    T: Error,
    E: Error,
{
    let time_after_load = Instant::now();

    let greachability_arena = GReachabilityArena::new();
    let sreachability_arena = SReachabilityArena::new();
    let mut syntactic_reachability =
        GReachability::new(&grammar, &greachability_arena, &tokens, gproc, info.max_len);
    let mut sreachability = SReachability::new(&sreachability_arena);

    let time_after_reachability_built = Instant::now();
    let mut time_after_find = Vec::new();

    let mut found_length = None;
    let mut outputs = None;
    for current_len in 0..=info.max_len {
        info!("Updating to length {}...", current_len);
        syntactic_reachability.update_until(current_len);
        let has_syn = if let Some(ref e) = syntactic_reachability.get_start_edges().get(current_len)
        {
            e.len() != 0
        } else {
            false
        };
        info!("Has syntactic reachability: {}", has_syn);
        match find(
            sproc,
            &sreachability_arena,
            &syntactic_reachability,
            current_len,
            current_len,
            &mut sreachability,
        ) {
            Some(start_edge) => {
                time_after_find.push(Instant::now());
                found_length = Some(current_len);
                let result = sreachability.generate_from(start_edge.ptr(), sproc, info.verbose_gen);
                if let Some(ref output) = info.output_name {
                    let mut output =
                        std::fs::File::create(output).expect("Unable to open output file.");
                    for token in result.iter() {
                        writeln!(output, "{}", token).unwrap();
                    }
                }
                outputs = Some(result);
                break;
            }
            None => {
                time_after_find.push(Instant::now());
            }
        }
    }

    Ok(FixTaskResult {
        time_before_load,
        time_after_load,
        time_after_reachability_built,
        time_after_find,
        found_length,
        outputs,
    })
}

mod do_fix_inner {
    pub trait DoFixInner {}
    impl DoFixInner for super::DoFixImpl {}
}

pub trait DoFix: do_fix_inner::DoFixInner {
    fn do_fix<'a, GProc, SProc, PG, T, E>(
        self,
        grammar: &'a Grammar<'a>,
        tokens: &Vec<Token<'a, '_>>,
        gproc: &GProc,
        sproc: &SProc,
        info: &FixTaskInfo,
        time_before_load: Instant,
    ) -> Result<FixTaskResult, FixTaskError<T, E>>
    where
        PG: UnionProp,
        GProc: GProcessor<PG = PG>,
        SProc: SProcessor<PG = PG>,
        T: Error,
        E: Error;
}
pub struct DoFixImpl;

impl DoFix for DoFixImpl {
    fn do_fix<'a, GProc, SProc, PG, T, E>(
        self,
        grammar: &'a Grammar<'a>,
        tokens: &Vec<Token<'a, '_>>,
        gproc: &GProc,
        sproc: &SProc,
        info: &FixTaskInfo,
        time_before_load: Instant,
    ) -> Result<FixTaskResult, FixTaskError<T, E>>
    where
        PG: UnionProp,
        GProc: GProcessor<PG = PG>,
        SProc: SProcessor<PG = PG>,
        T: Error,
        E: Error,
    {
        do_fix_impl(grammar, tokens, gproc, sproc, info, time_before_load)
    }
}

pub trait FixingInputProcessorBase {
    fn info(&self) -> &FixingInfo;
}

pub trait FixingInputProcessor: FixingInputProcessorBase {
    fn process<'a>(
        &self,
        grammar: &'a Grammar<'a>,
        input_str: &str,
        env_str: &str,
        info: &FixTaskInfo,
        time_before_load: Instant,
        do_fix: impl DoFix,
    ) -> Result<FixTaskResult, FixTaskError<Self::TokenizerError, Self::EnvLoadError>>;

    type TokenizerError: Error;
    type EnvLoadError: Error;
}

fn fix_in_loop<'a, P, T, E>(
    processor: &P,
    info: &FixTaskInfo,
    grammar: &'a Grammar<'a>,
) -> Result<FixTaskResult, FixTaskError<T, E>>
where
    P: FixingInputProcessor<TokenizerError = T, EnvLoadError = E>,
    T: Error,
    E: Error,
{
    let time_before_load = Instant::now();

    let input = fs::read_to_string(info.input_name.as_str())
        .map_err(|e| FixTaskError::ReadInputError(e))?;
    let env =
        fs::read_to_string(info.env_name.as_str()).map_err(|e| FixTaskError::ReadEnvError(e))?;

    processor.process(
        &grammar,
        input.as_str(),
        env.as_str(),
        &info,
        time_before_load,
        DoFixImpl,
    )
}

pub fn fix<P, T, E>(
    inputs: impl Iterator<Item = FixTaskInfo>,
    processor: &P,
) -> Vec<Result<FixTaskResult, FixTaskError<T, E>>>
where
    P: FixingInputProcessor<TokenizerError = T, EnvLoadError = E>,
    T: Error,
    E: Error,
{
    let grammar_arena = GrammarArena::new();
    let grammar = Grammar::new(&grammar_arena, processor.info().grammar).unwrap();
    let mut result = Vec::new();
    for info in inputs {
        let r = fix_in_loop(processor, &info, &grammar);
        match r {
            Ok(ref r) => {
                let time_load = r.time_after_load - r.time_before_load;
                let time_load = time_load.as_secs_f64();
                let time_build = r.time_after_reachability_built - r.time_after_load;
                let time_build = time_build.as_secs_f64();
                let time_find = match r.time_after_find.last() {
                    Some(x) => x.clone() - r.time_after_reachability_built,
                    None => Duration::new(0, 0),
                };
                let time_find = time_find.as_secs_f64();
                println!(
                    "---RESULT---,input_name:{},length:{},time_load:{},time_build:{},time_find:{}",
                    info.input_name,
                    match r.found_length {
                        Some(l) => l.to_string(),
                        None => "-1".to_string(),
                    },
                    time_load,
                    time_build,
                    time_find,
                );
            }
            Err(ref e) => {
                println!("---RESULT---,input_name:{},error:{:?}", info.input_name, e)
            }
        }

        result.push(r);
    }
    result
}
