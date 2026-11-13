//! Contains the assembler and disassembler

pub mod assemble;
pub mod disassemble;

mod errors;
mod expressions;
mod find_op_code;
mod label;
mod tokenizer;

use std::fs;

use crate::{
    cli::{AssembleArgs, DisassembleArgs},
    ecodes::{E_ASSEMBLER, E_DISASSEMBLER, E_IO_ERROR, E_SUCCESS},
};

use self::{assemble::Assembler, disassemble::disassemble_vec};

/// Run the `assemble` subcommand, returning the process exit code
pub fn run_assembler(args: AssembleArgs) -> i32 {
    let mut assembler = Assembler::new(args);
    let bytes = match assembler.assemble() {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("{}", e);
            return E_ASSEMBLER;
        }
    };
    match assembler.write(&bytes) {
        Ok(()) => E_SUCCESS,
        Err(e) => {
            eprintln!("Failed to write output: {}", e);
            E_IO_ERROR
        }
    }
}

macro_rules! ok_or_return {
    ($res:expr, $ret:expr) => {
        match $res {
            Ok(val) => val,
            Err(e) => {
                eprintln!("{}", e);
                return $ret;
            }
        }
    };
}

/// Run the `disassemble` subcommand, returning the process exit code
pub fn run_disassembler(args: DisassembleArgs) -> i32 {
    let bytes = ok_or_return!(fs::read(&args.input), E_IO_ERROR);
    let content = ok_or_return!(disassemble_vec(&bytes), E_DISASSEMBLER).join("\n");
    match args.output {
        Some(filename) => ok_or_return!(fs::write(filename, content), E_IO_ERROR),
        None => println!("{}", content),
    }
    E_SUCCESS
}

#[cfg(test)]
mod tests {
    use crate::{cli, util};

    use super::*;

    fn assemble(halt: bool, infile: &str, outfile: &str, exp: Vec<u8>) {
        let output = util::test::rsc(outfile);
        let r = run_assembler(cli::AssembleArgs {
            input: util::test::rsc(infile),
            output: output.clone(),
            hlt: halt,
            load_at: 0,
            no_register_definitions: false,
        });
        assert_eq!(r, E_SUCCESS);
        let out = fs::read(output).expect("file should exist");
        assert_eq!(out, exp);
    }

    #[test]
    fn assemble_simple_program() {
        assemble(
            true,
            "asm/simple.asm",
            "aux/simple.bin",
            vec![
                0x3e, 0xde, // MVI A, 0xde
                0x06, 0xad, // MVI B, 0xad
                0x80, // ADD B
                0x76, // HLT
            ],
        );
    }

    #[test]
    fn assemble_hello_world() {
        #[rustfmt::skip]
        let mut exp = vec![
            /*        */ 0x21, 0x10, 0x00, // LXI H _hello
            /* _do:   */ 0x7e,             // MOV A, M
            /*        */ 0xd3, 0x00,       // OUT 0
            /*        */ 0xfe, 0x00,       // CPI 0x00
            /*        */ 0xca, 0x0f, 0x00, // JZ _done
            /*        */ 0x23,             // INX H
            /*        */ 0xc3, 0x03, 0x00, // JMP _do
            /* _done: */ 0x76,             // HLT
        ];
        exp.append(&mut "hello world".as_bytes().to_vec());
        exp.push(0x00);
        assemble(false, "asm/hello-world.asm", "aux/hello-world.bin", exp);
    }

    #[test]
    fn registers_undefined_with_no_register_definitions() {
        let r = run_assembler(cli::AssembleArgs {
            input: util::test::rsc("asm/hello-world.asm"),
            output: util::test::rsc("aux/no-reg-defs.bin"),
            hlt: false,
            load_at: 0,
            no_register_definitions: true,
        });
        assert_eq!(r, E_ASSEMBLER, "H and M are unknown identifiers");
    }

    #[test]
    fn simple_program_disassemble() {
        let output = util::test::rsc("aux/simple.asm");
        let r = run_disassembler(cli::DisassembleArgs {
            input: util::test::rsc("bin/simple.bin"),
            output: Some(output.clone()),
        });
        assert_eq!(r, E_SUCCESS);
        let out = fs::read_to_string(output).expect("file should exist");
        assert_eq!(
            out.lines().collect::<Vec<_>>(),
            vec!["MVI A, 0xde", "MVI B, 0xad", "ADD B", "HLT"]
        );
    }
}
