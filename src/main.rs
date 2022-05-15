// TODO:
// - Documentation, properly, README too
// - Integration tests (somehow)
// - Log level CLI?
// - Disassemble with str indicator? (e.g. MOV L, B == 'h')

use clap::Parser;

use rs_8080::asm::{run_assembler, run_disassmbler};
use rs_8080::cli::{Cli, Commands};
use rs_8080::sys::run_system;

fn main() {
    env_logger::init();

    let args = Cli::parse();

    std::process::exit(match args.command {
        Commands::Run(subargs) => run_system(subargs),
        Commands::Assemble(subargs) => run_assembler(subargs),
        Commands::Disassemble(subargs) => run_disassmbler(subargs),
    });
}
