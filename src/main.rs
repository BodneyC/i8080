// TODO:
// - Documentation, properly, README too
// - Integration tests (somehow)
// - Log level CLI?

#[macro_use]
extern crate log;

use clap::Parser;

use assembler::{run_assembler, run_disassmbler};
use cli::{Cli, Commands};
use system::run_system;

mod assembler;
mod cli;
mod op_meta;
mod status_codes;
mod system;
mod util;

fn main() {
    env_logger::init();

    let args = Cli::parse();

    std::process::exit(match args.command {
        Commands::Run(subargs) => run_system(subargs),
        Commands::Assemble(subargs) => run_assembler(subargs),
        Commands::Disassemble(subargs) => run_disassmbler(subargs),
    });
}
