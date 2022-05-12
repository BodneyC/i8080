use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};

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
            add_hlt: true,
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

    i8080.run();

    // TODO: Should be threaded before `i8080.run()`
    if let Some(cons) = console {
        cons.run(); // Requires the HLT instruction (and to reach it...)
    }

    0
}

// vec![
//     0x3e, 0xde, // MVI A 0x3e
//     0x06, 0xad, // MVI B 0x06
//     0x80, // ADD B
//     0x76, // HLT
// ],
