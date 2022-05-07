#[derive(Debug, Clone, Copy)]
pub struct Label {
    pub value: Option<u16>,
    is_addr: bool,
    is_eq: bool,
    is_set: bool,
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

    pub fn new_eq(val: Option<u16>) -> Self {
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
