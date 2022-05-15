//! An Intel 8080 emulator and assembler
//!
//! # Examples
//!
//! Compile the ASM and include register to integer defs
//!  
//! ```sh
//! $ i8080 asm --register-definitions ./rsc/asm/hello-world.asm
//! ```
//!
//! Run the resulting binary
//!  
//! ```sh
//! $ i8080 run a.out
//! hello world
//! ```
//!
//! Or we can combine the two together
//!
//! ```sh
//! $ i8080 run --assemble ./rsc/asm/hello-world.asm
//! hello world
//! ```
//!
//! If assembling and running in one, register definitions are included and a `HLT` instruction is
//! placed at the end of the program.

pub mod asm;
pub mod cli;
pub mod ecodes;
pub mod sys;

mod meta;
mod util;

#[macro_use]
extern crate log;

use clap::Parser;

use asm::{run_assembler, run_disassmbler};
use cli::{Cli, Commands};
use sys::run_system;

fn main() {
    env_logger::init();

    let args = Cli::parse();

    std::process::exit(match args.command {
        Commands::Run(subargs) => run_system(subargs),
        Commands::Assemble(subargs) => run_assembler(subargs),
        Commands::Disassemble(subargs) => run_disassmbler(subargs),
    });
}
