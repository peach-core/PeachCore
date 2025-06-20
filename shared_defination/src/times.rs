#[derive(Clone)]
pub struct Tms {
    pub tms_systime: usize,
    pub tms_usrtime: usize,
    pub tms_child_systime: usize,
    pub tms_child_usrtime: usize,
}

impl Tms {
    pub fn new() -> Self {
        Self {
            tms_systime: 0,
            tms_usrtime: 0,
            tms_child_systime: 0,
            tms_child_usrtime: 0,
        }
    }
}
