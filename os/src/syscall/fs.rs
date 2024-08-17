const FD_STDOUT: usize = 1;
use log::*;
use crate::task::get_current_task_id;
use crate::config::{USER_STACK_SIZE, APP_SIZE_LIMIT};
use crate::loader::{get_base_i, get_user_stack_sp};

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let cur = get_current_task_id();
            let t = get_user_stack_sp(cur);
            let bs = get_base_i(cur);
            if (((buf as usize) >= t) && ((buf as usize) + len <= t + USER_STACK_SIZE))
                || (((buf as usize) >= bs) && (buf as usize) + len <= bs + APP_SIZE_LIMIT)
            {
                let slice = unsafe { core::slice::from_raw_parts(buf, len) };
                let str = core::str::from_utf8(slice).unwrap();
                print!("{}", str);
                len as isize
            } else {
                -1 as isize
            }
        }
        _ => {
            error!("Unsupported fd in sys_write!");
            -1 as isize
        }
    }
}
