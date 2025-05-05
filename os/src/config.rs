#![allow(unused)]
pub const TASK_STACK_SIZE: usize = 1024 * 8; // max stack size for task
pub const TASK_HEAP_SIZE: usize = 1024 * 8; // max heap size for task
pub const KERNEL_STACK_SIZE: usize = 1024 * 8; // max stack size for kernel
pub const KERNEL_HEAP_SIZE: usize = 4096 * 512; // max heap size for kernel

pub const KERNEL_THREAD_USER_STACK_TOP: usize = 0x0003_ffff_f000;
pub const KERNEL_THREAD_USER_STACK_BOTTOM: usize = KERNEL_THREAD_USER_STACK_TOP - TASK_STACK_SIZE;

pub const USER_STACK_TOP: usize = 0x0003_ffff_f000;
pub const USER_STACK_BOTTOM: usize = USER_STACK_TOP - TASK_STACK_SIZE;

pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SIZE_BITS: usize = 0xc;

pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
pub const TRAP_CONTEXT_BASE: usize = TRAMPOLINE - PAGE_SIZE;

pub use crate::board::{
    CLOCK_FREQ,
    MEMORY_END,
    MMIO,
};
