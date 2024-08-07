//! App management syscalls
use crate::batch::{run_next_app, print_current_app_info};

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    println!("[kernel] Application exited with code {}", exit_code);
    run_next_app()
}

/// exercise 2.01, print current app id & name
pub fn sys_get_taskinfo() -> isize {
    print_current_app_info();
    0
}