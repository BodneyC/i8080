use std::path::PathBuf;

use clap::{self, Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(name = "i8080", about = "A shitty I8080 emulator", long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Run(RunArgs),
    #[clap(visible_alias = "ass")]
    Assemble(AssembleArgs),
    #[clap(visible_alias = "dis")]
    Disassemble(DisassembleArgs),
}

#[derive(Debug, Args)]
#[clap(about = "Run the emulator")]
pub struct RunArgs {
    #[clap(help = "File to load into memory")]
    pub file: String,
    #[clap(long, help = "Positional file should be assembled")]
    pub from_asm: bool,
    #[clap(long, help = "Randomize flags and memory")]
    pub randomize: bool,
    #[clap(long, help = "Load program at given address")]
    pub load_at: Option<u16>,
    #[clap(long, help = "Disable the console device")]
    pub no_console: bool,
}

#[derive(Debug, Args)]
#[clap(about = "Assemble a file into a bin")]
pub struct AssembleArgs {
    #[clap(help = "ASM file to assemble")]
    pub input: PathBuf,
    #[clap(short, long, default_value = "a.out", help = "Output filename")]
    pub output: PathBuf,
    #[clap(
        long,
        default_value = "0",
        help = "Address at which the file will be loaded"
    )]
    pub load_address: u16,
}

#[derive(Debug, Args)]
#[clap(about = "Disassemble a file into ASM")]
pub struct DisassembleArgs {
    #[clap(help = "Bin file to disassemble")]
    pub infile: String,
    #[clap(short, long, default_value = "a.asm", help = "Output filename")]
    pub outfile: String,
}
