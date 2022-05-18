//! Contains the assembler and disassembler

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
    ecodes::{E_ASSEMBLER, E_DISASSEMBLER, E_IO_ERROR, E_SUCCESS},
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
    let v_u8 = ok_or_return!(read_file_to_vec_u8(args.input), E_IO_ERROR);
    let v_strings = ok_or_return!(disassemble_vec(&v_u8), E_DISASSEMBLER);
    let content = v_strings.join("\n");
    if let Some(filename) = args.output {
        let mut f = ok_or_return!(File::create(filename), E_IO_ERROR);
        ok_or_return!(write!(f, "{}", content), E_IO_ERROR);
    } else {
        println!("{}", content);
    }
    E_SUCCESS
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        io::{self, BufRead, BufReader, Read},
        path::Path,
    };

    use crate::{cli, util};

    use super::*;

    fn read_to_v8<P: AsRef<Path>>(filename: P) -> Result<Vec<u8>, io::Error> {
        let mut f = File::open(&filename)?;
        let metadata = fs::metadata(&filename)?;
        let mut buffer = vec![0; metadata.len() as usize];
        f.read_exact(&mut buffer)?;
        Ok(buffer)
    }

    fn assemble(halt: bool, infile: &str, outfile: &str, exp: Vec<u8>) {
        let output = util::test::rsc(outfile);
        let r = run_assembler(cli::AssembleArgs {
            input: util::test::rsc(infile),
            output: output.clone(),
            hlt: halt,
            load_at: 0,
            register_definitions: true,
        });
        assert_eq!(r, E_SUCCESS);
        let out = read_to_v8(output).expect("file should exist");
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

    fn read_to_v_string<P: AsRef<Path>>(filename: P) -> Result<Vec<String>, io::Error> {
        let file = File::open(filename)?;
        let buf = BufReader::new(file);
        Ok(buf
            .lines()
            .map(|l| l.expect("could not parse line"))
            .collect())
    }

    #[test]
    fn simple_program_disassemble() {
        let output = util::test::rsc("aux/simple.asm");
        let r = run_disassmbler(cli::DisassembleArgs {
            input: util::test::rsc("bin/simple.bin"),
            output: Some(output.clone()),
        });
        assert_eq!(r, E_SUCCESS);
        let out = read_to_v_string(output).expect("file should exist");
        assert_eq!(out, vec!["MVI A, 0xde", "MVI B, 0xad", "ADD B", "HLT",]);
    }
}
