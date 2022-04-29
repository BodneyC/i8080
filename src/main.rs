#[macro_use]
extern crate log;

use std::sync::mpsc::{self, Receiver, Sender};

use console_device::{ConsoleDevice, special_chars};

use crate::i8080::I8080;

mod console_device;
mod flags;
mod i8080;
mod instruction_meta;
mod memory;
mod registers;
mod util;

fn main() {
    env_logger::init();

    let (tx, rx): (Sender<u8>, Receiver<u8>) = mpsc::channel();

    let mut i8080 = I8080::new(None, Some(tx), special_chars::EOT);
    let mut console = ConsoleDevice::new(rx);

    i8080.load(
        0x00,
        vec![
            0x3e, 0xde, // MVI A 0x3e
            0x06, 0xad, // MVI B 0x06
            0x80, // ADD B
        ],
    );

    i8080.cycle();
    i8080.cycle();
    i8080.cycle();
}
