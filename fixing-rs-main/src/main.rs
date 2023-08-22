#![cfg_attr(nightly, feature(alloc_error_hook))]

#[macro_use]
extern crate fixing_rs_macros;
#[macro_use]
extern crate lalrpop_util;

#[cfg(test)]
pub mod tests;

pub mod c;
pub mod cmd;
pub mod fixing;
pub mod gensrc;
pub mod grammars;
pub mod mem_limit;
pub mod mj;

#[cfg(feature = "trace_memory")]
pub mod trace_mem;

fn main() {
    cmd::run();
}

#[ctor::ctor]
fn init() {
    env_logger::Builder::from_default_env()
        .filter(None, log::LevelFilter::Info)
        .init();
}
