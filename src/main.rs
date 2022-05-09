#[macro_use]
extern crate log;

use clap::Parser;

use cli::{Cli, Commands};
use assembler::run_assembler;
use system::run_system;

mod assembler;
mod cli;
mod op_meta;
mod system;

fn main() {
    env_logger::init();

    let args = Cli::parse();

    std::process::exit(match args.command {
        Commands::Run(subargs) => run_system(subargs),
        Commands::Assemble(subargs) => run_assembler(subargs),
        Commands::Disassemble(_) => 1,
    });
}
