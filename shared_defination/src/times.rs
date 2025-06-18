pub struct Tms {
    pub tms_systime: usize,
    pub tms_usrtime: usize,
    pub tms_child_systime: usize,
    pub tms_child_usrtime: usize,
}