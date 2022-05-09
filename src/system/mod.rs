use std::sync::mpsc::{self, Receiver, Sender};

use crate::cli::RunArgs;

use self::device::console_device::{special_chars, ConsoleDevice};
use self::device::TxDevice;
use self::i8080::I8080;

pub mod i8080;

mod device;
mod flags;
mod memory;
mod registers;
mod util;

pub fn run_system(args: RunArgs) -> i32 {
    // TODO: Handle `file`, `from_asm`, and loading

    let mut console: Option<ConsoleDevice> = None;

    let mut i8080 = if args.no_console {
        I8080::new(vec![], vec![])
    } else {
        let (tx, rx): (Sender<u8>, Receiver<u8>) = mpsc::channel();
        console = Some(ConsoleDevice::new(rx, false));
        let tx_device = TxDevice {
            tx,
            eot_byte: special_chars::EOT,
        };
        I8080::new(vec![], vec![tx_device])
    };

    if args.randomize {
        i8080.randomize();
    }

    i8080.load(
        0x00,
        vec![
            0x3e, 0xde, // MVI A 0x3e
            0x06, 0xad, // MVI B 0x06
            0x80, // ADD B
            0x76, // HLT
        ],
    );

    i8080.run();

    // TODO: Should be threaded before `i8080.run()`
    if let Some(cons) = console {
        cons.run(); // Requires the HLT instruction (and to reach it...)
    }

    0
}
