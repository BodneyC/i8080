use std::num::ParseIntError;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::{fs, thread};

use rustyline::error::ReadlineError;

use crate::assembler::parser::Assembler;
use crate::cli::{AssembleArgs, RunArgs};

use self::device::console_device::{special_chars, ConsoleDevice};
use self::device::TxDevice;
use self::i8080::I8080;

mod device;
mod flags;
mod i8080;
mod memory;
mod registers;
mod util;

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

    let program = if args.from_asm {
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
                return 1;
            }
        }
    } else {
        match fs::read(args.file.clone()) {
            Ok(bytes) => bytes,
            Err(e) => {
                println!("Failed to read file: {}\n\n{}", filename_plain, e);
                return 1;
            }
        }
    };

    i8080.load(load_address, program);

    let th_cons = if let Some(cons) = console {
        Some(thread::spawn(move || cons.run()))
    } else {
        None
    };

    if args.interactive {
        i8080.interactive = true;
        run_interactive(&mut i8080);
        i8080.halt();
    } else {
        i8080.run();
    }

    if let Some(th) = th_cons {
        th.join().unwrap();
    }

    0
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

fn run_interactive(i8080: &mut I8080) {
    let mut rl = rustyline::Editor::<()>::with_config(
        rustyline::Config::builder()
            .edit_mode(rustyline::EditMode::Vi)
            .build(),
    );

    if rl.load_history("history.txt").is_err() {
        debug!("No previous command line history");
    }

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
        let cmd = input.nth(0).unwrap_or("");
        // Input is an iterator, so the above consumes the first
        let args: Vec<&str> = input.collect();
        match cmd {
            "h" | "?" | "help" => prompt_help(),
            "c" | "cycle" => {
                if i8080.halted {
                    println!("CPU previously halted, breaking");
                    break;
                }
                i8080.cycle();
                println!("{}", i8080.current_state);
            }
            "f" | "flags" => println!("{}", i8080.debug_flags()),
            "r" | "registers" => println!("{}", i8080.debug_registers()),
            "m" | "mem" | "memory" => {
                if args.len() > 2 {
                    println!("Too many args: {:?}", args);
                    continue;
                }
                if args.len() <= 1 {
                    println!("Too few args: {:?}", args);
                    continue;
                }
                let addr = continue_on_err!(parse_number(args.get(0).unwrap()));
                let len = continue_on_err!(parse_number(args.get(1).unwrap()));
                println!("{:?}", i8080.get_slice(addr, len))
            }
            "e" | "exit" => break,
            "" => continue,
            s => println!("Unknown command: {}", s),
        }
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

fn prompt_help() {
    println!(
        r#"
h | ? | help) show this information
e | exit)     exit the prompt

c | cycle) cycle the cpu

f | flags)     print current state of flags
r | registers) print current state of registers

m | mem | memory) print values in memory
    u16: location
    u16: n bytes
"#
    )
}
