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

// TODO:
// - Documentation, properly, README too

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
