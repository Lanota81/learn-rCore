//! Implementation of syscalls
//!
//! The single entry point to all system calls, [`syscall()`], is called
//! whenever userspace wishes to perform a system call using the `ecall`
//! instruction. In this case, the processor raises an 'Environment call from
//! U-mode' exception, which is handled as one of the cases in
//! [`crate::trap::trap_handler`].
//!
//! For clarity, each single syscall is implemented as its own function, named
//! `sys_` then the name of the syscall. You can find functions like this in
//! submodules, and you should also implement syscalls this way.

#![allow(unused)]

const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_GET_TASKINFO: usize = 127;
const MAX_SYSCALL_NUM: usize = 201;
static mut SYSCALL_CNT: [usize; MAX_SYSCALL_NUM] = [0; MAX_SYSCALL_NUM];

/// only used in syscall
struct SyscallCounter {
    inner: RefCell<[usize; MAX_SYSCALL_NUM]>,
}

unsafe impl Sync for SyscallCounter {}

impl SyscallCounter {
    pub const fn new(val: [usize; MAX_SYSCALL_NUM]) -> Self {
        SyscallCounter {
            inner: RefCell::new(val),
        }
    }

    pub fn increase_cnt(&self, call_id: usize) -> isize {
        let mut t = self.inner.borrow_mut();
        t[call_id] += 1;
        0
    }

    pub fn check_cnt(&self, call_id: usize) -> usize {
        let t = self.inner.borrow_mut();
        t[call_id]
    }
}

static CALL_COUNTER: SyscallCounter = unsafe { SyscallCounter::new(SYSCALL_CNT) };

mod fs;
mod process;

use core::cell::RefCell;

use fs::*;
use process::*;

/// handle syscall exception with `syscall_id` and other arguments
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    CALL_COUNTER.increase_cnt(syscall_id);
    match syscall_id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_GET_TASKINFO => sys_get_taskinfo(),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
