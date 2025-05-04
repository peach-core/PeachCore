use riscv::register::sstatus::FS;

/// =============================================
/// ==============  fpu operation  ==============
/// =============================================
#[inline]
pub fn fpu_enable() {
    unsafe {
        riscv::register::sstatus::set_fs(FS::Initial);
    }
}

#[inline]
#[allow(unused)]
pub fn fpu_disable() {
    unsafe {
        riscv::register::sstatus::set_fs(FS::Off);
    }
}
