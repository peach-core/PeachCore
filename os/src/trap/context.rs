use riscv::register::sstatus::{
    self,
    Sstatus,
    SPP,
};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TrapContext {
    pub x: [usize; 32],      // reg[0..31].
    pub sstatus: Sstatus,    // CSR sstatus reg.
    pub sepc: usize,         // CSR sepc reg.
    pub kernel_satp: usize,  // kernel kernel satp(page table entry).
    pub kernel_sp: usize,    // kernel stack addr in kernel space.
    pub trap_handler: usize, // virtual addr of trap_handler in kernel space.
    pub fx: [usize; 32],     // FP register[0..31].
}

impl TrapContext {
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }
    pub fn app_init_context(
        entry_point: usize, user_sp: usize, kernel_satp: usize, kernel_sp: usize,
        trap_handler: usize,
    ) -> Self {
        let mut sstatus = sstatus::read();
        // set CPU privilege to User after trapping back
        sstatus.set_spp(SPP::User);
        let mut cx = Self {
            x: [0; 32],
            sstatus,
            sepc: entry_point,
            kernel_satp,
            kernel_sp,
            trap_handler,
            fx: [0; 32],
        };
        cx.set_sp(user_sp);
        cx
    }

    pub fn kpthread_init_context(
        entry_point: usize, user_sp: usize, kernel_satp: usize, kernel_sp: usize,
        trap_handler: usize,
    ) -> Self {
        let mut sstatus = sstatus::read();
        // set CPU privilege to User after trapping back
        sstatus.set_spp(SPP::Supervisor);
        sstatus.set_spie(true);
        let mut cx = Self {
            x: [0; 32],
            sstatus,
            sepc: entry_point,
            kernel_satp,
            kernel_sp,
            trap_handler,
            fx: [0; 32],
        };
        cx.set_sp(user_sp);
        cx
    }
}
