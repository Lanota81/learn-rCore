use crate::config::PAGE_SIZE;
use crate::mm::{virt2phys, VirtAddr};
use crate::task::{
    suspend_current_and_run_next,
    exit_current_and_run_next,
    mmap_in_task,
    munmap_in_task,
};
use crate::timer::get_time_us;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

pub fn sys_exit(exit_code: i32) -> ! {
    println!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    let virt = VirtAddr(_ts as usize);
    if let Some(phys) = virt2phys(virt) {
        let _us = get_time_us();
        let ts = usize::from(phys) as *mut TimeVal;
        unsafe {
            *ts = TimeVal {
                sec: _us / 1_000_000,
                usec: _us % 1_000_000,
            };
        }
        0
    } else { -1 }
}

pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    if prot & !7 != 0 || prot == 0 { return -1; }
    if start & (PAGE_SIZE - 1) != 0 { return -1; }
    let end = VirtAddr::from(start + len).ceil();
    let start = VirtAddr::from(start).floor();
    mmap_in_task(start, end, prot)
}

pub fn sys_munmap(start: usize, len: usize) -> isize {
    if start & (PAGE_SIZE - 1) != 0 { return -1; }
    let end = VirtAddr::from(start + len).ceil();
    let start = VirtAddr::from(start).floor();
    munmap_in_task(start, end)
}