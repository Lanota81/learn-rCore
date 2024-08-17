mod context;
mod switch;
mod task;

use crate::config::MAX_APP_NUM;
use crate::loader::{get_num_app, init_app_cx};
use crate::sync::UPSafeCell;
use crate::sbi::shutdown;
use crate::timer::{get_time_ms, get_time_us};
use lazy_static::*;
use log::*;
use task::{TaskControlBlock, TaskStatus};

pub use context::TaskContext;

pub struct TaskManager {
    num_app: usize,
    inner: UPSafeCell<TaskManagerInner>,
}

struct TaskManagerInner {
    tasks: [TaskControlBlock; MAX_APP_NUM],
    current_task: usize,
    /// timer for each task
    timer: usize,
}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut tasks = [TaskControlBlock {
            task_cx: TaskContext::zero_init(),
            task_status: TaskStatus::UnInit,
            user_time: 0,
            kernel_time: 0,
        }; MAX_APP_NUM];
        for i in 0..num_app {
            tasks[i].task_cx = TaskContext::goto_restore(init_app_cx(i));
            tasks[i].task_status = TaskStatus::Ready;
        }
        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                    timer: 0,
                })
            },
        }
    };
}

impl TaskManager {
    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.tasks[0];
        task0.task_status = TaskStatus::Running;
        let next_task_cx_ptr = &task0.task_cx as *const TaskContext;

        inner.start_timer();
        unsafe {
            debug!(
                "[kernel] First task running, id = {}, kernel stack ptr = {:#x}",
                0,
                (*next_task_cx_ptr).stack_pointer()
            );
        }
        drop(inner);
        let mut _unused = TaskContext::zero_init();
        // before this, we should drop local variables that must be dropped manually
        unsafe {
            __switch(&mut _unused as *mut TaskContext, next_task_cx_ptr);
        }
        panic!("unreachable in run_first_task!");
    }

    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.task_kernel_time();
        inner.tasks[current].task_status = TaskStatus::Ready;
    }

    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.task_kernel_time();
        inner.tasks[current].task_status = TaskStatus::Exited;
    }

    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
    }

    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[next].task_status = TaskStatus::Running;
            inner.current_task = next;
            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
            if next != current {
                debug!(
                    "[kernel] Task {} {}, kernel stack pointer = {:#x}",
                    current,
                    match inner.tasks[current].task_status {
                        TaskStatus::Ready => "suspended",
                        TaskStatus::Exited => "exited",
                        _ => "has a status exception",
                    },
                    inner.tasks[current].task_cx.stack_pointer()
                );
                if inner.tasks[current].task_status == TaskStatus::Exited {
                    debug!(
                        "[kernel] Task {} has run for kernel_time: {} ms, user_time: {} ms",
                        current, inner.tasks[current].kernel_time, inner.tasks[current].user_time
                    );
                }
                debug!(
                    "[kernel] Task {} is running, kernel stack pointer = {:#x}",
                    next,
                    inner.tasks[next].task_cx.stack_pointer()
                );
            }
            drop(inner);
            // before this, we should drop local variables that must be dropped manually
            unsafe {
                __switch(current_task_cx_ptr, next_task_cx_ptr);
            }
            // go back to user mode
        } else {
            let inner = self.inner.exclusive_access();
            let current = inner.current_task;
            debug!(
                "[kernel] Task {} exited, run for kernel_time: {} ms, user_time: {} ms",
                current, inner.tasks[current].kernel_time, inner.tasks[current].user_time
            );
            println!("All applications completed!");
            unsafe {
                debug!("Switch tasks costs {} us", SWITCH_TIME_COUNTER);
            }
            shutdown();
        }
    }
}

impl TaskManagerInner {
    fn task_kernel_time(&mut self) {
        let start = self.timer;
        self.timer = get_time_ms();
        self.tasks[self.current_task].kernel_time += self.timer - start;
    }

    fn task_user_time(&mut self) {
        let start = self.timer;
        self.timer = get_time_ms();
        self.tasks[self.current_task].user_time += self.timer - start;
    }

    fn start_timer(&mut self) {
        self.timer = get_time_ms();
    }
}

pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}

/// update current task kernel timer
pub fn task_kernel_time() {
    TASK_MANAGER.inner.exclusive_access().task_kernel_time();
}

/// update current task user timer
pub fn task_user_time() {
    TASK_MANAGER.inner.exclusive_access().task_user_time();
}

static mut SWITCH_TIMER: usize = 0;
static mut SWITCH_TIME_COUNTER: usize = 0;
/// count usage of __switch
unsafe fn __switch(current_task_cx_ptr: *mut TaskContext, next_task_cx_ptr: *const TaskContext) {
    SWITCH_TIMER = get_time_us();
    switch::__switch(current_task_cx_ptr, next_task_cx_ptr);
    SWITCH_TIME_COUNTER += get_time_us() - SWITCH_TIMER;
}
