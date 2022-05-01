use crate::cli::AssembleArgs;

use self::assembler::Assembler;

pub mod assembler;
pub mod label;

mod find_op_code;

pub fn run_assembler(args: AssembleArgs) -> i32 {
    let mut assembler = Assembler::new();
    match assembler.assemble(args.input) {
        Ok(bytes) => match assembler.write_file(args.output, bytes) {
            Ok(_) => 0,
            Err(e) => {
                error!("Unable to write compiled file: {}", e);
                1
            }
        },
        Err(e) => {
            error!("Unable to write compiled file: {}", e);
            1
        }
    }
}
