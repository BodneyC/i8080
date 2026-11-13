//! Console printing device for the emulator
//!
//! The console device takes a receiver and ideally the emulator receives the counterpart
//! transmitter.
//!
//! Operation of the device is done through special characters. For example, the text passed to
//! it is buffered, the null or ETB characters are used to "flush" the buffer and print the
//! contents to the screen.
//!
//! Similarly end-of-transmission (EOT) is used to close operation of the device

use std::sync::mpsc::Receiver;

/// ASCII control characters understood by the device
pub mod special_chars {
    pub const NUL: u8 = 0x00;
    pub const EOT: u8 = 0x04;
    pub const ETB: u8 = 0x17;
    pub const BEL: u8 = 0x07;
    pub const BS: u8 = 0x08;
    pub const CR: u8 = 0x0d;
    pub const LF: u8 = 0x0a;
}

pub struct ConsoleDevice {
    rx: Receiver<u8>,
}

impl ConsoleDevice {
    pub fn new(rx: Receiver<u8>) -> Self {
        Self { rx }
    }

    /// Process received bytes until EOT, or the transmitter is dropped
    ///
    /// Returns any unflushed contents of the buffer
    pub fn run(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![];
        let mut idx: usize = 0;
        for byte in self.rx.iter() {
            debug!("Console device received [{}]", byte);
            match byte {
                special_chars::EOT => break,
                // End of transmission block and null terminator, flush the buffer
                special_chars::ETB | special_chars::NUL => {
                    println!("{}", String::from_utf8_lossy(&buf));
                    buf.clear();
                    idx = 0;
                }
                // Device should immediately acknowledge
                special_chars::BEL => println!(":BEL:"),
                special_chars::BS => {
                    if idx > 0 {
                        idx -= 1;
                        buf.remove(idx);
                    }
                }
                // Carriage return, overwrite from the start of the current line
                special_chars::CR => {
                    idx = buf[..idx]
                        .iter()
                        .rposition(|b| *b == special_chars::LF)
                        .map_or(0, |lf| lf + 1);
                }
                _ => {
                    if idx == buf.len() {
                        buf.push(byte);
                    } else {
                        buf[idx] = byte;
                    }
                    idx += 1;
                }
            }
        }
        buf
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use super::*;

    fn run_console(input: &[u8]) -> String {
        let (tx, rx) = mpsc::channel();
        for byte in input.iter().chain(&[special_chars::EOT]) {
            tx.send(*byte).expect("receiver should be alive");
        }
        String::from_utf8_lossy(&ConsoleDevice::new(rx).run()).into_owned()
    }

    #[test]
    fn simple_string() {
        assert_eq!(run_console(b"Hello"), "Hello");
    }

    #[test]
    fn newline_string() {
        assert_eq!(run_console(b"Hello\nthere"), "Hello\nthere");
    }

    #[test]
    fn newline_with_cr() {
        assert_eq!(run_console(b"Hello\nthe\rre"), "Hello\nree");
    }

    #[test]
    fn repeated_cr() {
        assert_eq!(run_console(b"ab\ncd\r\rX"), "ab\nXd");
    }

    #[test]
    fn backspace() {
        assert_eq!(run_console(b"Hello\x08scape"), "Hellscape");
    }

    #[test]
    fn dropped_transmitter_stops_device() {
        let (tx, rx) = mpsc::channel();
        tx.send(b'a').unwrap();
        drop(tx);
        assert_eq!(ConsoleDevice::new(rx).run(), b"a");
    }
}
