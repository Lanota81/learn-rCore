use crate::mm::{
    UserBuffer,
    translated_byte_buffer,
    translated_refmut,
    translated_str,
};
use crate::task::{current_task, current_task_pid, current_user_token, get_task_by_pid, is_valid_addr, Post, BUF_LEN};
use crate::fs::{make_pipe, OpenFlags, open_file};
use alloc::sync::Arc;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
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
        file.write(
            UserBuffer::new(translated_byte_buffer(token, buf, len))
        ) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.read(
            UserBuffer::new(translated_byte_buffer(token, buf, len))
        ) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(
        path.as_str(),
        OpenFlags::from_bits(flags).unwrap()
    ) {
        let mut inner = task.inner_exclusive_access();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

pub fn sys_pipe(pipe: *mut usize) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let mut inner = task.inner_exclusive_access();
    let (pipe_read, pipe_write) = make_pipe();
    let read_fd = inner.alloc_fd();
    inner.fd_table[read_fd] = Some(pipe_read);
    let write_fd = inner.alloc_fd();
    inner.fd_table[write_fd] = Some(pipe_write);
    *translated_refmut(token, pipe) = read_fd;
    *translated_refmut(token, unsafe { pipe.add(1) }) = write_fd;
    0
}

pub fn sys_dup(fd: usize) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
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
pub fn sys_mailread(buf: *mut u8, len: usize) -> isize {
    if !is_valid_addr(buf as usize) {
        return -1;
    }
    let task = current_task().unwrap();
    let token = current_user_token();
    let inner = task.inner_exclusive_access();
    let mut mailbox = inner.mailbox.exclusive_access();
    if len == 0 {
        return if mailbox.readable() { 0 } else { -1 };
    }
    let len = len.min(BUF_LEN);
    if let Some(p) = mailbox.fetch() {
        p.read(&mut UserBuffer::new(translated_byte_buffer(
            token, buf, len,
        )))
    } else {
        -1
    }
}

pub fn sys_mailwrite(pid: usize, buf: *mut u8, len: usize) -> isize {
    if !is_valid_addr(buf as usize) {
        return -1;
    }
    let token = current_user_token();
    if pid == current_task_pid() {
        let task = current_task().unwrap();
        let inner = task.inner_exclusive_access();
        let mut mailbox = inner.mailbox.exclusive_access();
        if len == 0 {
            return if mailbox.writable() { 0 } else { -1 };
        }

        let len = len.min(BUF_LEN);
        if mailbox.push(&Arc::new(Post::new(UserBuffer::new(
            translated_byte_buffer(token, buf, len),
        )))) != -1 {
            len as isize
        } else {
            -1
        }
    } else if let Some(task) = get_task_by_pid(pid) {
        let inner = task.inner_exclusive_access();
        let mut mailbox = inner.mailbox.exclusive_access();
        if len == 0 {
            return if mailbox.writable() { 0 } else { -1 };
        }
        
        let len = len.min(BUF_LEN);
        if mailbox.push(&Arc::new(Post::new(UserBuffer::new(
            translated_byte_buffer(token, buf, len),
        )))) != -1 {
            len as isize
        } else {
            -1
        }
    } else {
        -1
    }
}
