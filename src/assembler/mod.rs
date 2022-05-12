use crate::cli::AssembleArgs;

use self::parser::Assembler;

pub mod parser;

mod errors;
mod expressions;
mod find_op_code;
mod label;
mod tokenizer;
mod util;

pub fn run_assembler(args: AssembleArgs) -> i32 {
    let output = args.output.clone();
    let mut assembler = Assembler::new(args);
    match assembler.assemble() {
        Ok(bytes) => match assembler.write_file(bytes) {
            Ok(_) => 0,
            Err(e) => {
                println!("{}\n {}", e, output.as_path().display(),);
                1
            }
        },
        Err(_) => 1,
    }
}
