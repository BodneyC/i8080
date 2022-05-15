mod common;

use rs_8080::{asm::run_assembler, asm::run_disassmbler, cli, ecodes::E_SUCCESS};

fn assemble(halt: bool, infile: &str, outfile: &str, exp: Vec<u8>) {
    let output = common::rsc(outfile);
    let r = run_assembler(cli::AssembleArgs {
        input: common::rsc(infile),
        output: output.clone(),
        hlt: halt,
        load_at: 0,
        register_definitions: true,
    });
    assert_eq!(r, E_SUCCESS);
    let out = common::read_to_v8(output).expect("file should exist");
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
fn simple_program_disassemble() {
    let output = common::rsc("aux/simple.asm");
    let r = run_disassmbler(cli::DisassembleArgs {
        input: common::rsc("bin/simple.bin"),
        output: Some(output.clone()),
    });
    assert_eq!(r, E_SUCCESS);
    let out = common::read_to_v_string(output).expect("file should exist");
    assert_eq!(out, vec!["MVI A, 0xde", "MVI B, 0xad", "ADD B", "HLT",]);
}
