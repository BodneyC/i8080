//! Label container specifying the type of value/instruction the label relates to

/// How a label was defined, which determines whether it may later be redefined
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelKind {
    /// Marks the address of the line it is attached to
    Address,
    /// Defined by `EQU`, immutable
    Equ,
    /// Defined by `SET`, may be redefined by a later `SET`
    Set,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Label {
    pub value: u16,
    pub kind: LabelKind,
}

impl Label {
    pub fn new_addr(value: u16) -> Self {
        Self {
            value,
            kind: LabelKind::Address,
        }
    }

    pub fn new_equ(value: u16) -> Self {
        Self {
            value,
            kind: LabelKind::Equ,
        }
    }

    pub fn new_set(value: u16) -> Self {
        Self {
            value,
            kind: LabelKind::Set,
        }
    }
}
