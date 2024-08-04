#![no_std]
#![no_main]
mod lang_items;
mod sbi;

#[macro_use]
mod console;
mod logging;

use core::{arch::global_asm /*, error */ };
use log::*;
// use sbi_rt::Extension;

global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub fn rust_main() -> ! {
    extern "C" {
        fn stext();
        fn etext();
        fn srodata();
        fn erodata();
        fn sdata();
        fn edata();
        fn sbss();
        fn ebss();
        fn boot_stack_lower_bound();
        fn boot_stack_top();
    }

    clear_bss();
    logging::init();
    println!("[Kernel] Hello, world!");
    trace!("[Kernel] .text [{:#x}, {:#x})", 
        stext as usize, etext as usize);
    debug!("[Kernel] .rodata [{:#x}, {:#x})", 
        srodata as usize, erodata as usize);
    info!("[Kernel] .data [{:#x}, {:#x})", 
        sdata as usize, edata as usize);
    warn!("[Kernel] boot_stack top=bottom={:#x}, lower_bound={:#x}", 
        boot_stack_top as usize, boot_stack_lower_bound as usize);
    error!("[Kernel] .bss [{:#x}, {:#x})", 
        sbss as usize, ebss as usize);
    
    sbi::shutdown(false)
}

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| {
        unsafe { (a as *mut u8).write_volatile(0) }
    });
}
