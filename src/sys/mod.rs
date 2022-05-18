//! System and emulator
//!
//! I've probably mucked up in a few places but I believe this to be a fairly true-ish emulator.
//!
//! # Input/Output
//!
//! The 8080 instruction set includes two instructions, `IN` and `OUT`, for reading from an IO
//! device to the accumulator and writing to the accumulator from an IO device.
//!
//! I've provided a simplistic console device which can be used to output text to make use of this
//! `OUT` instruction.
//!
//! # Interrupts
//!
//! Interrupts may be issued as single `u8` operation codes as per the manual I found somewhere;
//! currently this is only use in interactive mode as the `i` command performs this function.
//!
//! # Interactive
//!
//! The emulator can be ran interactively, this will drop you into a prompt where you can
//! cycling/debug the CPU.

pub mod i8080;

mod device;
mod flags;
mod memory;
mod registers;

use std::{
    fs,
    num::ParseIntError,
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use rustyline::error::ReadlineError;

use crate::{
    asm::{assemble::Assembler, disassemble::disassemble_instruction},
    cli::{AssembleArgs, RunArgs},
    ecodes::{E_ASSEMBLER, E_IO_ERROR, E_SUCCESS},
};

use self::{
    device::console_device::{special_chars, ConsoleDevice},
    device::TxDevice,
    i8080::I8080,
};

pub fn run_system(args: RunArgs) -> i32 {
    let mut console: Option<ConsoleDevice> = None;

    let mut i8080 = if args.no_console {
        I8080::new(vec![], vec![])
    } else {
        let (tx, rx): (Sender<u8>, Receiver<u8>) = mpsc::channel();
        console = Some(ConsoleDevice::new(rx, false));
        let tx_device = TxDevice::new(tx, special_chars::EOT);
        I8080::new(vec![], vec![tx_device])
    };

    if args.randomize {
        i8080.randomize();
    }

    let load_address = args.load_at.unwrap_or(0);
    let filename_plain = args.file.as_path().display();

    let program = if args.assemble {
        let mut assembler = Assembler::new(AssembleArgs {
            input: args.file.clone(),
            output: PathBuf::new(),
            load_at: load_address,
            register_definitions: true,
            hlt: true,
        });
        match assembler.assemble() {
            Ok(bytes) => bytes,
            Err(_) => {
                return E_ASSEMBLER;
            }
        }
    } else {
        match fs::read(args.file.clone()) {
            Ok(bytes) => bytes,
            Err(e) => {
                println!("Failed to read file: {}\n\n{}", filename_plain, e);
                return E_IO_ERROR;
            }
        }
    };

    i8080.load(load_address, program);

    let th_cons = console.map(|cons| thread::spawn(move || cons.run()));

    if args.interactive {
        i8080.interactive = true;
        run_interactive(&mut i8080);
        i8080.halt();
    } else {
        i8080.run(args.emulate_clock_speed);
    }

    if let Some(th) = th_cons {
        th.join().unwrap();
    }

    E_SUCCESS
}

macro_rules! continue_on_err {
    ($res:expr) => {
        match $res {
            Ok(val) => val,
            Err(e) => {
                println!("Couldn't parse arg\n{}", e);
                continue;
            }
        }
    };
}

const PROMPT_HELP: &str = "\
h | ? | help)        show this information
q | quit | e | exit) exit the prompt
c | cycle)           cycle the cpu
s | sys | system)    print flags and registers

i | int | interrupt) issue interrupt
    u8: op code

d | dis | disassemble) disassemble next instruction
    u16: address [default: PC]

m | mem | memory) print values in memory
    u16: n bytes [default: 1]
    u16: address [default: PC]\
";

fn run_interactive(i8080: &mut I8080) {
    let mut rl = rustyline::Editor::<()>::with_config(
        rustyline::Config::builder()
            .edit_mode(rustyline::EditMode::Vi)
            .build(),
    );

    if rl.load_history("history.txt").is_err() {
        debug!("No previous command line history");
    }

    let mut cycling = false;

    loop {
        let raw_input = match rl.readline("> ") {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                line
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                println!("Exiting...");
                break;
            }
            Err(e) => {
                println!("Error: {:?}", e);
                break;
            }
        };
        let mut input = raw_input.split_whitespace();
        let cmd = input.next().unwrap_or("").to_lowercase();
        // Input is an iterator, so the above consumes the first
        let args: Vec<&str> = input.collect();
        match cmd.as_str() {
            "c" | "cycle" | "" => {}
            _ => cycling = false,
        }
        match cmd.as_str() {
            "h" | "?" | "help" => println!("{}", PROMPT_HELP),
            "c" | "cycle" => {
                cycling = true;
                if !prompt_cycle(i8080) {
                    break;
                }
            }
            "s" | "sys" | "system" => println!("{}", i8080.describe_system()),
            "i" | "int" | "interrupt" => {
                if args.len() != 1 {
                    println!("Interrupt takes one arg");
                    continue;
                }
                let inst = continue_on_err!(parse_number(args.get(0).unwrap())) as u8;
                i8080.issue_interrupt(inst);
                println!("Interrupt issues, instruction {:#02x}", inst);
            }
            "m" | "mem" | "memory" => {
                if args.len() > 2 {
                    println!("Up to two args required: {:?}", args);
                    continue;
                }
                let len = continue_on_err!(parse_number(args.get(0).unwrap_or(&"1")));
                let addr = if let Some(arg) = args.get(1) {
                    continue_on_err!(parse_number(arg))
                } else {
                    i8080.get_pc()
                };
                println!("{:#04x} {:02x?}", addr, i8080.get_memory_slice(addr, len))
            }
            "d" | "dis" | "disassemble" => {
                if args.len() > 1 {
                    println!("Zero or one args required: {:?}", args);
                    continue;
                }
                let addr = if let Some(arg) = args.get(0) {
                    continue_on_err!(parse_number(arg))
                } else {
                    i8080.get_pc()
                };
                let (s, _) =
                    continue_on_err!(disassemble_instruction(&i8080.get_memory_slice(addr, 3), 0));
                println!("{}", s);
            }
            "q" | "quit" | "e" | "exit" => break,
            "" => {
                if cycling {
                    if !prompt_cycle(i8080) {
                        break;
                    }
                } else {
                    continue;
                }
            }
            s => println!("Unknown command: {}", s),
        }
    }
}

fn prompt_cycle(i8080: &mut I8080) -> bool {
    if i8080.halted {
        println!("CPU previously halted, breaking");
        false
    } else {
        i8080.cycle();
        println!("{}", i8080.current_state);
        true
    }
}

fn parse_number(input: &str) -> Result<u16, ParseIntError> {
    let mut s = input.to_string();
    let radix = if s.starts_with("0x") {
        16
    } else if s.starts_with("0b") {
        2
    } else if s.starts_with("0o") {
        8
    } else {
        10
    };
    if radix != 10 {
        s = s[2..].to_string();
    }
    u16::from_str_radix(&s, radix)
}
