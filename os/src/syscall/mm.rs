use crate::task::current_process;

pub fn sys_brk(size: isize) -> isize {
    let process = current_process();
    if let Some(old_brk) = process.change_program_brk(size) {
        old_brk as isize
    }
    else {
        -1
    }
}

pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    let process = current_process();
    process.current_task_mmap(start, len, prot)
}

pub fn sys_munmap(start: usize) -> isize {
    let process = current_process();
    process.current_task_munmap(start)
}
