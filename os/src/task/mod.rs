mod context;
mod switch;
mod task;
mod manager;
mod processor;
mod pid;
mod mail;

use crate::loader::get_app_data_by_name;
use crate::mm::{VirtAddr, VirtPageNum, PageTable};
use switch::__switch;
use task::{TaskControlBlock, TaskStatus};
use alloc::sync::Arc;
use manager::fetch_task;
use lazy_static::*;

pub use context::TaskContext;
pub use processor::{
    run_tasks,
    current_task,
    current_user_token,
    current_trap_cx,
    take_current_task,
    schedule,
    current_task_pid,
};
pub use manager::{add_task, get_task_by_pid};
pub use pid::{PidHandle, pid_alloc, KernelStack};
pub use mail::{Post, Mail, BUF_LEN};

pub fn suspend_current_and_run_next() {
    // There must be an application running.
    let task = take_current_task().unwrap();

    // ---- access current TCB exclusively
    let mut task_inner = task.inner_exclusive_access();
    let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
    // Change status to Ready
    task_inner.task_status = TaskStatus::Ready;
    drop(task_inner);
    // ---- release current PCB

    // push back to ready queue.
    add_task(task);
    // jump to scheduling cycle
    schedule(task_cx_ptr);
}

pub fn exit_current_and_run_next(exit_code: i32) {
    // take from Processor
    let task = take_current_task().unwrap();
    // **** access current TCB exclusively
    let mut inner = task.inner_exclusive_access();
    // Change status to Zombie
    inner.task_status = TaskStatus::Zombie;
    // Record exit code
    inner.exit_code = exit_code;
    // do not move to its parent but under initproc

    // ++++++ access initproc TCB exclusively
    {
        let mut initproc_inner = INITPROC.inner_exclusive_access();
        for child in inner.children.iter() {
            child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC));
            initproc_inner.children.push(child.clone());
        }
    }
    // ++++++ release parent PCB

    inner.children.clear();
    // deallocate user space
    inner.memory_set.recycle_data_pages();
    drop(inner);
    // **** release current PCB
    // drop task manually to maintain rc correctly
    drop(task);
    // we do not have to save task context
    let mut _unused = TaskContext::zero_init();
    schedule(&mut _unused as *mut _);
}

lazy_static! {
    pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new(
        TaskControlBlock::new(get_app_data_by_name("initproc").unwrap())
    );
}

pub fn add_initproc() {
    add_task(INITPROC.clone());
}

pub fn mmap_in_task(start: VirtPageNum, end: VirtPageNum, prot: usize) -> isize {
    current_task().unwrap().mmap_in_task(start, end, prot)
}

pub fn munmap_in_task(start: VirtPageNum, end: VirtPageNum) -> isize {
    current_task().unwrap().munmap_in_task(start, end)
}

pub fn is_valid_addr(addr: usize) -> bool {
    let token = current_user_token();
    let virt = VirtAddr::from(addr);
    let vpn = VirtPageNum::from(virt.floor());

    if let Some(_t) = PageTable::from_token(token)
        .translate(vpn) {
            true
        } else {
            false
        }
}