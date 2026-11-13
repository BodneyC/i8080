//! Splits a line of assembly into its label, instruction, arguments, and comment
//!
//! The syntax is simple enough that no proper grammar is used, the general form of a line is:
//!
//! ```asm
//! [LABEL:] [INSTRUCTION [ARG[, ARG...]]] [; COMMENT]
//! ```
//!
//! Everything outside of string literals is made uppercase.

use super::{
    errors::{OpParseError, ParserError},
    find_op_code,
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LineMeta {
    pub raw_line: String,
    pub line_no: usize,
    pub comment: Option<String>,
    /// Instruction, meta-instruction, or macro name; `None` only when `label_only`
    pub inst: Option<String>,
    pub args_list: Vec<String>,
    pub label: Option<String>,
    /// An estimated op-code; the true op-code is only known once arguments are resolved
    ///
    /// `None` implies the instruction is a macro invocation
    pub op_code: Option<u16>,
    pub label_only: bool,
    /// Absolute address for program lines, offset from the start of the macro for macro lines
    pub address: u16,
    pub width: usize,
}

impl LineMeta {
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

/// Byte index of the first `target` in `s` which isn't inside a string literal
fn find_unquoted(s: &str, target: char) -> Option<usize> {
    let mut in_quotes = false;
    let mut escaped = false;
    for (idx, c) in s.char_indices() {
        if in_quotes {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '\'' {
                in_quotes = false;
            }
        } else if c == '\'' {
            in_quotes = true;
        } else if c == target {
            return Some(idx);
        }
    }
    None
}

fn validate_label(label: &str) -> Result<(), ParserError> {
    let mut chars = label.chars();
    let valid_start = chars.next().is_some_and(|c| c == '_' || c.is_alphabetic());
    if valid_start && chars.all(|c| c == '_' || c.is_alphanumeric()) {
        Ok(())
    } else {
        Err(ParserError::InvalidLabel(label.to_string()))
    }
}

/// Split comma-separated arguments, uppercasing everything outside of string literals
fn split_args(args: &str) -> Result<Vec<String>, ParserError> {
    let mut args_list: Vec<String> = vec![];
    let mut this_arg = String::new();
    let mut in_quotes = false;
    let mut escaped = false;

    for c in args.chars() {
        if in_quotes {
            this_arg.push(c);
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '\'' {
                in_quotes = false;
            }
        } else if c == ',' {
            args_list.push(std::mem::take(&mut this_arg));
        } else {
            in_quotes = c == '\'';
            this_arg.extend(c.to_uppercase());
        }
    }

    if in_quotes {
        return Err(ParserError::UnterminatedString(args.to_string()));
    }
    args_list.push(this_arg);

    let args_list: Vec<String> = args_list.iter().map(|a| a.trim().to_string()).collect();
    if args_list.iter().any(String::is_empty) {
        return Err(ParserError::EmptyArgument);
    }
    Ok(args_list)
}

/// Tokenize a single line of assembly
///
/// Returns `None` for lines containing nothing but whitespace and/or a comment
pub fn tokenize(raw_line: &str) -> Result<Option<LineMeta>, ParserError> {
    let mut line = raw_line.trim();

    let mut comment: Option<String> = None;
    if let Some(comment_idx) = find_unquoted(line, ';') {
        comment = Some(line[comment_idx + 1..].trim().to_string());
        line = line[..comment_idx].trim();
    }

    if line.is_empty() {
        return Ok(None);
    }

    let mut label: Option<String> = None;
    if let Some(label_idx) = find_unquoted(line, ':') {
        let label_str = line[..label_idx].trim().to_uppercase();
        validate_label(&label_str)?;
        label = Some(label_str);
        line = line[label_idx + 1..].trim();
    }

    if line.is_empty() {
        return Ok(Some(LineMeta::label_only(
            label,
            comment,
            raw_line.to_string(),
        )));
    }

    let (inst, args_list) = match line.split_once(char::is_whitespace) {
        Some((inst, args)) => (inst, split_args(args.trim())?),
        None => (line, vec![]),
    };
    let inst = inst.to_uppercase();

    // Expression resolution is required to know the true op code, however all op codes of the
    // same instruction are the same width which we can use to work out label addressing for
    // expression parsing
    //
    // E.g. MOV M, A is one byte, as is MOV A, A
    let op_code = match find_op_code::from_args(&inst, 0, 0) {
        Ok(op) => Some(op as u16),
        // Could be a macro, this is checked by the assembler
        Err(OpParseError::NoSuchInstruction(_)) => None,
        Err(e) => return Err(e.into()),
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
        let line = "MOV 'fish', B".to_string();
        let meta = tokenize(&line).expect("tokenizer failed");
        assert!(meta.is_some(), "should be a valid line");
        let meta = meta.unwrap();
        assert_eq!(meta.args_list, vec!["'fish'", "B"], "args");
    }

    #[test]
    fn string_arg_with_comma() {
        let line = "MOV 'fish, other fish', B".to_string();
        let meta = tokenize(&line).expect("tokenizer failed");
        assert!(meta.is_some(), "should be a valid line");
        let meta = meta.unwrap();
        assert_eq!(meta.args_list, vec!["'fish, other fish'", "B"], "args");
    }

    #[test]
    fn string_arg_with_other_phrases() {
        let line = "MOV XOR 'fish' + 1, B".to_string();
        let meta = tokenize(&line).expect("tokenizer failed");
        assert!(meta.is_some(), "should be a valid line");
        let meta = meta.unwrap();
        assert_eq!(meta.args_list, vec!["XOR 'fish' + 1", "B"], "args");
    }

    #[test]
    fn labels_insts_and_args_are_made_uppercase() {
        let line = "some_label: mov xor 'fish', b ; lowercase".to_string();
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

    #[test]
    fn delimiters_in_strings() {
        let meta = tokenize("_l: CPI ';' ; real comment")
            .expect("tokenizer failed")
            .unwrap();
        assert_eq!(meta.label, Some("_L".to_string()));
        assert_eq!(meta.args_list, vec!["';'"]);
        assert_eq!(meta.comment, Some("real comment".to_string()));

        let meta = tokenize("DB 'a:b', 'it\\'s'")
            .expect("tokenizer failed")
            .unwrap();
        assert_eq!(meta.label, None);
        assert_eq!(meta.args_list, vec!["'a:b'", "'it\\'s'"]);
    }

    #[test]
    fn non_ascii_string_keeps_last_arg() {
        let meta = tokenize("DB 'héllo', 0")
            .expect("tokenizer failed")
            .unwrap();
        assert_eq!(meta.args_list, vec!["'héllo'", "0"]);
    }

    #[test]
    fn tab_separated_instruction() {
        let meta = tokenize("MOV\tA, B").expect("tokenizer failed").unwrap();
        assert_eq!(meta.inst, Some("MOV".to_string()));
        assert_eq!(meta.args_list, vec!["A", "B"]);
    }

    #[test]
    fn empty_and_unterminated_args() {
        let e = tokenize("DB 1,").expect_err("trailing comma");
        assert!(matches!(e, ParserError::EmptyArgument));
        let e = tokenize("DB 'abc").expect_err("unterminated string");
        assert!(matches!(e, ParserError::UnterminatedString(_)));
        let e = tokenize(": NOP").expect_err("empty label");
        assert!(matches!(e, ParserError::InvalidLabel(_)));
        let e = tokenize("1abc: NOP").expect_err("label starting with a digit");
        assert!(matches!(e, ParserError::InvalidLabel(_)));
    }
}
