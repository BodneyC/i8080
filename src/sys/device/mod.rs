//! Simplistic IO device
//!
//! The emulator can use IO devices with its `IN` and `OUT` instructions, so the `IN` will be used
//! to receive data (`rx`) and `OUT` will be used to transmit (`tx`) data.
//!
//! The devices here take to relevant opposites to the above.

pub mod console_device;

use std::sync::mpsc::{Receiver, Sender};

pub struct TxDevice {
    pub tx: Sender<u8>,
    pub eot_byte: u8,
}

impl TxDevice {
    pub fn new(tx: Sender<u8>, eot_byte: u8) -> Self {
        Self { tx, eot_byte }
    }
}

pub struct RxDevice {
    pub rx: Receiver<u8>,
}

impl RxDevice {
    #[allow(dead_code)] // This may be used at some point...
    pub fn new(rx: Receiver<u8>) -> Self {
        Self { rx }
    }
}
