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
    pub fn new(rx: Receiver<u8>) -> Self {
        Self { rx }
    }
}
