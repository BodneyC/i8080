use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

// use crate::assembler::find_op_code;
use crate::errors::{AssemblerError, ParserError};

pub struct Instruction {
    addr: u16,
    arg0: u32,
    arg1: u32,
    comment: String,
    inst: String,
    op_code: u8,
    psw: bool,
    raw_args: String,
    sp: bool,
}

// let addr = 0;
// let labels = hashmap::new();
// let instructions = Vec::new();
// read-file {
//   for line in f {
//      let (label, inst, arg0, arg1) = line.split();
//      let width = find_op_code::from_args(inst, 0, 0).width;
//      if (addr as u32 + width as u32 > 0xffff) {
//          Err(OOM)
//      }
//      addr += width;
//      labels[label] = addr
//      instructions.push(Inst { inst, arg0, arg1, addr })
//   }
// }
// for inst in instructions {
//    let (arg0, arg1) = parse_args(inst.raw_args) {
//      let (arg0, arg1) = args.split(',')
//
//    }
// }

pub struct Assembler {
    labels: HashMap<String, u16>,
    instructions: Vec<Instruction>,
    bytes: Vec<u8>,
}

impl Assembler {
    pub fn new() -> Self {
        Self {
            labels: HashMap::new(),
            instructions: Vec::new(),
            bytes: Vec::new(),
        }
    }

    // Very side-effect-y, I know
    pub fn assemble(&mut self, input: PathBuf) -> Result<(), AssemblerError> {
        self.parse_file(input)
    }

    pub fn write_file(&self, output: PathBuf) -> Result<(), io::Error> {
        fs::write(output, &self.bytes)
    }

    fn parse_file(&mut self, input: PathBuf) -> Result<(), AssemblerError> {
        match read_lines(input) {
            Ok(lines) => {
                for (line_no, line_res) in lines.enumerate() {
                    if let Ok(line) = line_res {
                        if let Err(e) = self.parse_line(line) {
                            error!("{}", e); // Give file lineno and stuff
                            return Err(AssemblerError::ParseError(e));
                        }
                    }
                }
                Ok(())
            }
            Err(ioe) => Err(AssemblerError::FileReadError(ioe)),
        }
    }

    fn parse_line(&mut self, line_string: String) -> Result<(), ParserError> {
        let mut line = line_string.trim();
        let mut comment = String::new();
        if let Some(comment_idx) = line.find(";") {
            comment.push_str(&line[comment_idx + 1..].trim());
            line = &line[0..comment_idx].trim();
        }
        Ok(())
    }
}

fn read_lines<P: AsRef<Path>>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>> {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
