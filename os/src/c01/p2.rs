#![allow(unused)]
use core::arch::asm;

pub fn print_stack_trace_pointer_chain() {
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

fn test_call_print_stack3() {
    print_stack_trace_pointer_chain();
}

fn test_call_print_stack2() {
    test_call_print_stack3();
}

fn test_call_print_stack1() {
    test_call_print_stack2();
}

pub fn test_call_print_stack() {
    test_call_print_stack1();
}
