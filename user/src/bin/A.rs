#![no_std]
#![no_main]

extern crate user_lib;

use core::*;
use core::arch::asm;
use user_lib::println;

#[no_mangle]
fn main() {
    println!("=== Stack Trace from fp chain ===\n");

    let mut fp: usize = 0;
    unsafe {
        asm!("mv {0}, fp", inout(reg) fp);
    }

    while fp > 0 {
        println!("Current fp: {:#x}", fp);
        println!("Return Address: {:#x}", unsafe {
            *((fp - 8) as *const usize)
        });
        let t: usize = unsafe { *((fp - 16) as *const usize) };
        println!("Old stack pointer: {:#x}", t);
        println!("");

        fp = t;
    }
    println!("=== End Trace ===");
}