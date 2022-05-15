//! Cli argument structures
//!
//! This cli is based on subcommands, each with its own sub-options
//!
//! - `run` to access the emulator
//! - `assemble` to access the assembler
//! - `disassemble` to access the disassembler
//!
//! Use the `--help` option for each subcommand to find out more...

use std::path::PathBuf;

use clap::{self, Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(name = "i8080", about = "An I8080 emulator", long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Run(RunArgs),
    #[clap(visible_alias = "asm")]
    Assemble(AssembleArgs),
    #[clap(visible_alias = "dis")]
    Disassemble(DisassembleArgs),
}

#[derive(Debug, Args)]
#[clap(about = "Run the emulator")]
pub struct RunArgs {
    #[clap(help = "File to load into memory")]
    pub file: PathBuf,
    #[clap(short, long, help = "Load program at given address")]
    pub load_at: Option<u16>,
    #[clap(short, long, help = "Randomize registers and memory")]
    pub randomize: bool,
    #[clap(short, long, help = "Run the emulator in a prompt")]
    pub interactive: bool,
    #[clap(short, long, help = "Provided file requires assembly")]
    pub assemble: bool,
    #[clap(long, help = "Disable the console device")]
    pub no_console: bool,
    #[clap(long, help = "Sleep occasionally to match 2HZ")]
    pub emulate_clock_speed: bool,
}

#[derive(Debug, Args)]
#[clap(about = "Assemble a file into a bin")]
pub struct AssembleArgs {
    #[clap(help = "ASM file to assemble")]
    pub input: PathBuf,
    #[clap(short, long, default_value = "a.out", help = "Output filename")]
    pub output: PathBuf,
    #[clap(long, help = "Add a HLT instruction at the end of the program")]
    pub hlt: bool,

    #[clap(
        long,
        default_value = "0",
        help = "Address at which the file will be loaded"
    )]
    pub load_at: u16,

    #[clap(
        long,
        visible_alias = "reg-defs",
        help = "Include register EQU statements"
    )]
    pub register_definitions: bool,
}

#[derive(Debug, Args)]
#[clap(about = "Disassemble a file into ASM")]
pub struct DisassembleArgs {
    #[clap(help = "Bin file to disassemble")]
    pub input: PathBuf,
    #[clap(short, long, help = "Output filename")]
    pub output: Option<PathBuf>,
}
