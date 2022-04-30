use std::sync::mpsc::Receiver;

/// I8080 to take an rx of one channel and a tx of another DeviceController to take the
/// corresponding channel halves
///
/// out => tx.write(self.registers.a)
/// in => self.registers.a = rx.read()

// I don't know another way to namespace some consts... maybe bad practice but who can really tell
// these days
pub mod special_chars {
    pub const EOT: u8 = 0x04;
    pub const ETB: u8 = 0x17;
    pub const BEL: u8 = 0x07;
    pub const BS: u8 = 0x08;
    pub const CR: u8 = 0x0d;
    pub const LF: u8 = 0x0a;
}

pub struct ConsoleDevice {
    rx: Receiver<u8>,
    echo: bool,
}

impl ConsoleDevice {
    pub fn new(rx: Receiver<u8>, echo: bool) -> Self {
        Self { rx, echo }
    }

    pub fn run(&self) -> Vec<u8> {
        match self.echo {
            true => self.run_echo(),
            false => self.run_no_echo(),
        }
    }

    fn run_echo(&self) -> Vec<u8> {
        unimplemented!();
    }

    fn run_no_echo(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![];
        let mut idx: usize = 0;
        loop {
            if let Ok(byte) = self.rx.recv() {
                match byte {
                    // EOT, end of tranmission
                    special_chars::EOT => break,
                    // ETB, end of tranmission block (using as flush)
                    special_chars::ETB => {
                        println!("{}", String::from_utf8_lossy(&buf));
                        buf.clear();
                        idx = 0;
                    }
                    // BEL, device should immediately acknowledge
                    special_chars::BEL => println!(":BEL:"),
                    // BS, backspace
                    special_chars::BS => {
                        if idx > 0 {
                            idx -= 1;
                            buf.remove(idx);
                        }
                    }
                    // CR, overwrite from previous
                    special_chars::CR => {
                        if let Some(rev_last_lf) = buf[0..idx]
                            .iter()
                            .rev()
                            .position(|b| *b == special_chars::LF)
                        {
                            idx = buf.len() - rev_last_lf;
                        } else {
                            idx = 0;
                        }
                    }
                    // Printables
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
        }
        buf
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::{self, SendError, Sender};

    use super::*;

    #[test]
    fn simple_string() {
        let (tx, rx): (Sender<u8>, Receiver<u8>) = mpsc::channel();

        let act = "Hello".to_owned();

        let mut vec = act.as_bytes().to_vec();
        vec.push(special_chars::EOT);

        let mut err: Option<SendError<u8>> = None;
        for byte in vec.iter() {
            if let Err(e) = tx.send(*byte) {
                err = Some(e);
                break;
            }
        }

        assert!(err.is_none(), "{:?}", err.unwrap());

        let console = ConsoleDevice { rx, echo: false };

        assert_eq!(act, String::from_utf8_lossy(&(console.run())));
    }

    #[test]
    fn newline_string() {
        let (tx, rx): (Sender<u8>, Receiver<u8>) = mpsc::channel();

        let act = "Hello\nthere".to_owned();

        let mut vec = act.as_bytes().to_vec();
        vec.push(special_chars::EOT);

        let mut err: Option<SendError<u8>> = None;
        for byte in vec.iter() {
            if let Err(e) = tx.send(*byte) {
                err = Some(e);
                break;
            }
        }

        assert!(err.is_none(), "{:?}", err.unwrap());

        let console = ConsoleDevice { rx, echo: false };

        assert_eq!(act, String::from_utf8_lossy(&(console.run())));
    }

    #[test]
    fn newline_with_cr() {
        let (tx, rx): (Sender<u8>, Receiver<u8>) = mpsc::channel();

        let act = "Hello\nthe\rre".to_owned();
        let exp = "Hello\nree";

        let mut vec = act.as_bytes().to_vec();
        vec.push(special_chars::EOT);

        let mut err: Option<SendError<u8>> = None;
        for byte in vec.iter() {
            if let Err(e) = tx.send(*byte) {
                err = Some(e);
                break;
            }
        }

        assert!(err.is_none(), "{:?}", err.unwrap());

        let console = ConsoleDevice { rx, echo: false };

        assert_eq!(exp, String::from_utf8_lossy(&(console.run())));
    }

    #[test]
    fn backspace() {
        let (tx, rx): (Sender<u8>, Receiver<u8>) = mpsc::channel();

        let mut vec = "Hello".as_bytes().to_vec();
        vec.push(0x08);
        vec.append(&mut "scape".as_bytes().to_vec());
        vec.push(special_chars::EOT);

        let mut err: Option<SendError<u8>> = None;
        for byte in vec.iter() {
            if let Err(e) = tx.send(*byte) {
                err = Some(e);
                break;
            }
        }

        assert!(err.is_none(), "{:?}", err.unwrap());

        let console = ConsoleDevice { rx, echo: false };

        assert_eq!("Hellscape", String::from_utf8_lossy(&(console.run())));
    }
}
