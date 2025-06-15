use crate::{
    fs::{
        OpenFlags,
        make_pipe,
        open_file,
    },
    mm::{
        UserBuffer,
        translated_byte_buffer,
        translated_refmut,
        translated_str,
    },
    task::{
        current_process,
        current_user_token,
    },
};
use alloc::sync::Arc;

use super::user_space::__user;

pub fn sys_write(fd: usize, buf: __user<*const u8>, len: usize) -> isize {
    let token = current_user_token();
    let process = current_process();
    let inner = process.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.write(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: __user<*const u8>, len: usize) -> isize {
    let token = current_user_token();
    let process = current_process();
    let inner = process.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        log::error!("sys_read: fd {} out of bounds", fd);
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            log::error!("sys_read: file {} is not readable", fd);
            return -1;
        }
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        log::error!("sys_read: fd {} is not open", fd);
        -1
    }
}

pub fn sys_openat(dirfd: i32, path: __user<*const u8>, flags: i32) -> isize {
    const AT_FDCWD: i32 = -100;

    let token = current_user_token();
    let path_str = translated_str(token, path);
    let path_str = path_str.strip_prefix("./").unwrap_or(&path_str);
    let process = current_process();
    let open_flags = OpenFlags::from_bits(flags).unwrap_or(OpenFlags::empty());

    let file_opt = if path_str.starts_with("/") || dirfd == AT_FDCWD {
        open_file(path_str, open_flags)
    } else {
        // absolute path
        let inner = process.inner_exclusive_access();
        if (dirfd as usize) >= inner.fd_table.len() || inner.fd_table[dirfd as usize].is_none() {
            log::error!("sys_openat: dirfd {} is invalid", dirfd);
            None
        } else {
            let dir_file = inner.fd_table[dirfd as usize].as_ref().unwrap().clone();
            drop(inner);
            // TODO: handle relative path
            open_file(path_str, open_flags)
        }
    };

    if let Some(file) = file_opt {
        let mut inner = process.inner_exclusive_access();
        for (i, slot) in inner.fd_table.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(file);
                return i as isize;
            }
        }
        inner.fd_table.push(Some(file));
        (inner.fd_table.len() - 1) as isize
    } else {
        log::error!("sys_openat: failed to open file {}", path_str);
        -1
    }
}
pub fn sys_close(fd: usize) -> isize {
    let process = current_process();
    let mut inner = process.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

pub fn sys_pipe(pipe: __user<*mut usize>) -> isize {
    let process = current_process();
    let token = current_user_token();
    let mut inner = process.inner_exclusive_access();
    let (pipe_read, pipe_write) = make_pipe();
    let read_fd = inner.alloc_fd();
    inner.fd_table[read_fd] = Some(pipe_read);
    let write_fd = inner.alloc_fd();
    inner.fd_table[write_fd] = Some(pipe_write);
    *translated_refmut(token, pipe) = read_fd;
    *translated_refmut(token, unsafe { pipe.inner().add(1).into() }) = write_fd;
    0
}

pub fn sys_dup(fd: usize) -> isize {
    let process = current_process();
    let mut inner = process.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    let new_fd = inner.alloc_fd();
    inner.fd_table[new_fd] = Some(Arc::clone(inner.fd_table[fd].as_ref().unwrap()));
    new_fd as isize
}
