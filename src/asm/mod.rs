pub mod assemble;
pub mod disassemble;

mod errors;
mod expressions;
mod find_op_code;
mod label;
mod tokenizer;

use std::fs::File;
use std::io::Write;

use crate::{
    cli::{AssembleArgs, DisassembleArgs},
    status_codes::{E_ASSEMBLER, E_DISASSEMBLER, E_IO_ERROR, E_SUCCESS},
    util::read_file_to_vec_u8,
};

use self::{assemble::Assembler, disassemble::disassemble_vec};

pub fn run_assembler(args: AssembleArgs) -> i32 {
    let output = args.output.clone();
    let mut assembler = Assembler::new(args);
    match assembler.assemble() {
        Ok(bytes) => match assembler.write(bytes) {
            Ok(_) => E_SUCCESS,
            Err(e) => {
                println!("{}\n {}", e, output.as_path().display(),);
                E_IO_ERROR
            }
        },
        Err(_) => E_ASSEMBLER,
    }
}

macro_rules! ok_or_return {
    ($res:expr, $ret:expr) => {
        match $res {
            Ok(val) => val,
            Err(e) => {
                println!("{}", e);
                return $ret;
            }
        }
    };
}

pub fn run_disassmbler(args: DisassembleArgs) -> i32 {
    let v_u8 = ok_or_return!(read_file_to_vec_u8(args.infile), E_IO_ERROR);
    let v_strings = ok_or_return!(disassemble_vec(&v_u8), E_DISASSEMBLER);
    let content = v_strings.join("\n");
    if let Some(filename) = args.outfile {
        let mut f = ok_or_return!(File::create(filename), E_IO_ERROR);
        ok_or_return!(write!(f, "{}", content), E_IO_ERROR);
    } else {
        println!("{}", content);
    }
    E_SUCCESS
}
