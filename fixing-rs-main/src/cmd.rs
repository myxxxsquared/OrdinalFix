use crate::{fixing::FixCmd, gensrc::GenSrcCmd, mem_limit::limit_memory};
use clap::{Parser, Subcommand};
use log::error;
use std::panic::catch_unwind;

#[derive(Parser)]
pub struct MainCmd {
    #[command(subcommand)]
    cmd: Cmd,

    #[arg(long)]
    memory_limit: Option<usize>,
}

#[derive(Subcommand)]
pub enum Cmd {
    Fix(FixCmd),
    GenSrc(GenSrcCmd),
}

impl Cmd {
    pub fn run(self) {
        match self {
            Cmd::Fix(fix_cmd) => fix_cmd.run(),
            Cmd::GenSrc(gen_src_cmd) => gen_src_cmd.run(),
        }
    }
}

pub fn run() {
    run_cmd(MainCmd::parse());
}

pub fn run_cmd(cmd: MainCmd) {
    if let Some(memory_limit) = cmd.memory_limit {
        if memory_limit != 0 {
            limit_memory(memory_limit).unwrap();
        }
    }

    let result = catch_unwind(move || {
        cmd.cmd.run();
    });

    match result {
        Ok(_) => {}
        Err(p) => {
            if let Some(s) = p.downcast_ref::<String>() {
                error!("PANIC:{}", s);
            } else if let Some(s) = p.downcast_ref::<&'static str>() {
                error!("PANIC:{}", s);
            } else {
                error!("PANIC:Unknown error");
            }
        }
    }
}
