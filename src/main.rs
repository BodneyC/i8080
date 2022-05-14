// TODO:
// - Documentation, properly, README too
// - Integration tests (somehow)
// - Log level CLI?

#[macro_use]
extern crate log;

pub mod asm;
pub mod cli;
pub mod sys;

mod op_meta;
mod status_codes;
mod util;

use clap::Parser;

use crate::asm::{run_assembler, run_disassmbler};
use crate::cli::{Cli, Commands};
use crate::sys::run_system;

fn main() {
    env_logger::init();

    let args = Cli::parse();

    std::process::exit(match args.command {
        Commands::Run(subargs) => run_system(subargs),
        Commands::Assemble(subargs) => run_assembler(subargs),
        Commands::Disassemble(subargs) => run_disassmbler(subargs),
    });
}
