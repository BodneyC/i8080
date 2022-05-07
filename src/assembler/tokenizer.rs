use super::errors::{OpParseError, ParserError};
use super::find_op_code;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineMeta {
    pub raw_line: String,
    pub line_no: usize,
    pub comment: Option<String>,
    pub inst: Option<String>,
    pub args_list: Vec<String>,
    pub label: Option<String>,
    pub op_code: Option<u8>,
    pub label_only: bool,
    pub address: u16,
    pub width: usize,
    pub uses_pc: bool,
}

impl Default for LineMeta {
    fn default() -> Self {
        Self {
            raw_line: String::new(),
            line_no: 0,
            comment: None,
            inst: None,
            args_list: vec![],
            label: None,
            op_code: None,
            label_only: false,
            address: 0,
            width: 0,
            uses_pc: false,
        }
    }
}

impl LineMeta {
    pub fn erroring(line: &LineMeta) -> Self {
        trace!("@{} | {}", line.line_no, line.raw_line);
        Self {
            line_no: line.line_no,
            raw_line: line.raw_line.to_string(),
            ..Default::default()
        }
    }

    pub fn label_only(label: Option<String>, comment: Option<String>, raw_line: String) -> Self {
        Self {
            label,
            comment,
            raw_line,
            label_only: true,
            ..Default::default()
        }
    }
}

// A more robust system wouldn't be a bad idea, but as the syntax is fairly simple might as
// well do it fairly simply
pub fn tokenize(raw_line: &String) -> Result<Option<LineMeta>, ParserError> {
    let mut line = raw_line.trim();
    let mut comment: Option<String> = None;
    // Check for a comment in the line
    if let Some(comment_idx) = line.find(";") {
        if line.len() > comment_idx {
            comment = Some(line[comment_idx + 1..].trim().to_string());
        }
        line = line[..comment_idx].trim();
    }

    // If no line remains, tokenize is still successful, but no LineMeta is returned
    if line.len() == 0 {
        return Ok(None);
    }

    let mut label: Option<String> = None;
    // Check if some label precedes the instruction
    if let Some(label_idx) = line.find(":") {
        let label_str = line[..label_idx].trim().to_uppercase();
        if label_str.len() == 0 {
            return Err(ParserError::InvalidLabel(label_str));
        }
        for c in label_str.chars() {
            if c != '_' && !c.is_alphabetic() {
                return Err(ParserError::InvalidLabel(label_str));
            }
        }
        label = Some(label_str);
        if line.len() > label_idx {
            line = line[label_idx + 1..].trim();
        } else {
            line = "";
        }
    }

    // If no line remains, line is just a marker for a label
    if line.len() == 0 {
        return Ok(Some(LineMeta::label_only(
            label,
            comment,
            raw_line.to_string(),
        )));
    }
    let mut raw_args: Option<String> = None;
    // Look for the space between instruction and args
    if let Some(args_idx) = line.find(" ") {
        // No bounds check here as we trim before
        raw_args = Some(line[args_idx + 1..].trim().to_string());
        line = line[..args_idx].trim();
    }

    let mut args_list: Vec<String> = vec![];
    // If any args exists...
    if let Some(args) = raw_args {
        let mut in_quotes: bool = false;
        let mut char_escaped: bool = false;
        let mut this_arg: String = String::new();
        for (idx, c) in args.chars().enumerate() {
            // Start or stop "being" in a string unless escaped
            if c == '\'' && !char_escaped {
                in_quotes = !in_quotes;
            }
            // If we are escaping this character, unset it
            if char_escaped {
                char_escaped = false;
            }
            // Only escape inside a string
            if c == '\\' && in_quotes {
                char_escaped = true;
            }
            // is_escaped can only be set inside a string
            if c == ',' && !in_quotes {
                args_list.push(this_arg.trim().to_string());
                this_arg = String::new();
            } else {
                // Push to this arg before checking end of string
                if in_quotes {
                    this_arg.push(c);
                } else {
                    this_arg.push(c.to_uppercase().nth(0).unwrap_or(c));
                }
            }
            if idx == args.len() - 1 {
                if in_quotes {
                    return Err(ParserError::UnterminatedString(args));
                }
                args_list.push(this_arg.trim().to_string());
            }
        }
    }

    // Expression resolution is required to know the true op code, however all op codes of the
    // same instruction are the same width which we can use to work out label addressing for
    // expression parsing
    //
    // E.g. MOV M, A is one byte, as is MOV A, A
    //
    // It is this expression parsing that will give us the true arguments and then the true
    // operation code... in short, `op_code` is a temporary value...
    //
    // *Note*: The NoSuchInstruction could be caused by a macro...
    let op_code: Option<u8>;
    let inst = line.to_uppercase();
    match find_op_code::from_args(inst.as_str(), 0, 0) {
        Ok(op) => op_code = Some(op as u8),
        Err(e) => match e {
            OpParseError::NoSuchInstruction(_) => op_code = None,
            _ => {
                return Err(ParserError::NoInstructionFound(e));
            }
        },
    };

    Ok(Some(LineMeta {
        comment,
        inst: Some(inst),
        args_list,
        raw_line: raw_line.to_string(),
        label,
        op_code,
        ..Default::default()
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_line() {
        let empty_line = "".to_owned();
        let meta = tokenize(&empty_line).expect("tokenizer failed");
        assert_eq!(None, meta, "Empty string should be a None token");
    }

    #[test]
    fn mov() {
        let line = "MOV".to_owned();
        let meta = tokenize(&line).expect("tokenizer failed");
        assert!(meta.is_some(), "should be a valid line");
        let meta = meta.unwrap();
        assert!(!meta.label_only, "not label only");
        assert!(meta.comment.is_none(), "has no comment");
        assert_eq!(meta.raw_line, line, "raw line");
        assert_eq!(meta.op_code, Some(0x40), "op code first MOV");
        assert_eq!(meta.args_list, Vec::<String>::new(), "args");
    }

    #[test]
    fn mov_a_b() {
        let line = "MOV A, B".to_owned();
        let meta = tokenize(&line).expect("tokenizer failed");
        assert!(meta.is_some(), "should be a valid line");
        let meta = meta.unwrap();
        assert!(!meta.label_only, "not label only");
        assert!(meta.comment.is_none(), "has no comment");
        assert_eq!(meta.raw_line, line, "raw line");
        assert_eq!(meta.op_code, Some(0x40), "op code first MOV");
        assert_eq!(meta.args_list, vec!["A", "B"], "args");
    }

    #[test]
    fn label_and_comment_with_line() {
        let label = "SOME_LABEL".to_string();
        let comment = "comment".to_string();
        let line = format!("{}: MOV A, B, C ; {}", label, comment);
        let meta = tokenize(&line).expect("tokenizer failed");
        assert!(meta.is_some(), "should be a valid line");
        let meta = meta.unwrap();
        assert!(!meta.label_only, "not label only");
        assert_eq!(meta.label, Some(label), "label");
        assert_eq!(meta.comment, Some(comment), "comment");
        assert_eq!(meta.raw_line, line, "not raw line");
        assert_eq!(meta.op_code, Some(0x40), "op code first MOV");
        assert_eq!(meta.args_list, vec!["A", "B", "C"], "args");
    }

    #[test]
    fn label_and_comment_only() {
        let label = "SOME_LABEL".to_string();
        let comment = "comment".to_string();
        let line = format!("{}:;{}", label, comment);
        let meta = tokenize(&line).expect("tokenizer failed");
        assert!(meta.is_some(), "should be a valid line");
        let meta = meta.unwrap();
        assert!(meta.label_only, "should be label only");
        assert_eq!(meta.label, Some(label), "label");
        assert_eq!(meta.comment, Some(comment), "comment");
    }

    #[test]
    fn space_in_label() {
        let label = "SOME LABEL".to_string();
        let line = format!("{}: MOV", label);
        let meta = tokenize(&line).expect_err("label with space not permitted");
        assert!(matches!(meta, ParserError::InvalidLabel(_)));
    }

    #[test]
    fn string_arg() {
        let line = format!("MOV 'fish', B");
        let meta = tokenize(&line).expect("tokenizer failed");
        assert!(meta.is_some(), "should be a valid line");
        let meta = meta.unwrap();
        assert_eq!(meta.args_list, vec!["'fish'", "B"], "args");
    }

    #[test]
    fn string_arg_with_comma() {
        let line = format!("MOV 'fish, other fish', B");
        let meta = tokenize(&line).expect("tokenizer failed");
        assert!(meta.is_some(), "should be a valid line");
        let meta = meta.unwrap();
        assert_eq!(meta.args_list, vec!["'fish, other fish'", "B"], "args");
    }

    #[test]
    fn string_arg_with_other_phrases() {
        let line = format!("MOV XOR 'fish' + 1, B");
        let meta = tokenize(&line).expect("tokenizer failed");
        assert!(meta.is_some(), "should be a valid line");
        let meta = meta.unwrap();
        assert_eq!(meta.args_list, vec!["XOR 'fish' + 1", "B"], "args");
    }

    #[test]
    fn labels_insts_and_args_are_made_uppercase() {
        let line = format!("some_label: mov xor 'fish', b ; lowercase");
        let meta = tokenize(&line).expect("tokenizer failed");
        assert!(meta.is_some(), "should be a valid line");
        let meta = meta.unwrap();
        assert!(!meta.label_only, "not label only");
        assert_eq!(meta.label, Some("SOME_LABEL".to_string()), "label");
        assert_eq!(meta.comment, Some("lowercase".to_string()), "comment");
        assert_eq!(meta.raw_line, line, "not raw line");
        assert_eq!(meta.op_code, Some(0x40), "op code first MOV");
        assert_eq!(meta.inst, Some("MOV".to_string()), "op code first MOV");
        assert_eq!(meta.args_list, vec!["XOR 'fish'", "B"], "args");
    }

    // strings
}
