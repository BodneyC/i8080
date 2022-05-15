//! Label container specifying the type of value/instruction the label relates to

#[derive(Debug, Clone, Copy)]
pub struct Label {
    pub value: Option<u16>,
    pub is_addr: bool,
    pub is_eq: bool,
    pub is_set: bool,
}

impl Label {
    pub fn new_addr(val: Option<u16>) -> Self {
        Self {
            value: val,
            is_addr: true,
            is_eq: false,
            is_set: false,
        }
    }

    pub fn new_equ(val: Option<u16>) -> Self {
        Self {
            value: val,
            is_addr: false,
            is_eq: true,
            is_set: false,
        }
    }

    pub fn new_set(val: Option<u16>) -> Self {
        Self {
            value: val,
            is_addr: false,
            is_eq: false,
            is_set: true,
        }
    }
}
