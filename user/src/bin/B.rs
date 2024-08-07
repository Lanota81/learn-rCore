#![no_std]
#![no_main]

extern crate user_lib;

use user_lib::get_task_info;

#[no_mangle]
fn main() {
    get_task_info();
}