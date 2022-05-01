#[derive(Debug, Clone, Copy)]
pub struct Label {
    val: u16,
    is_eq: bool,
    is_set: bool,
}

impl Label {
    pub fn new(val: u16) -> Self {
        Self {
            val,
            is_eq: false,
            is_set: false,
        }
    }

    pub fn new_eq(val: u16) -> Self {
        Self {
            val,
            is_eq: true,
            is_set: false,
        }
    }

    pub fn new_set(val: u16) -> Self {
        Self {
            val,
            is_eq: false,
            is_set: true,
        }
    }
}
