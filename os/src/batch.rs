#![allow(unused)]
use lazy_static::*;
use crate::trap::TrapContext;
use crate::sync::UPSafeCell;
use core::ops::Add;
use core::str;
use core::arch::asm;
use riscv::use_sv32;

const USER_STACK_SIZE: usize = 4096;
const KERNEL_STACK_SIZE: usize = 4096 * 2;
const MAX_APP_NUM: usize = 16;
const APP_BASE_ADDRESS: usize = 0x80400000;
const APP_SIZE_LIMIT: usize = 0x20000;

#[repr(align(4096))]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

#[repr(align(4096))]
struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

static KERNEL_STACK: KernelStack = KernelStack { data: [0; KERNEL_STACK_SIZE] };
static USER_STACK: UserStack = UserStack { data: [0; USER_STACK_SIZE] };

impl KernelStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }
    pub fn push_context(&self, cx: TrapContext) -> &'static mut TrapContext {
        let cx_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        unsafe { *cx_ptr = cx; }
        unsafe { cx_ptr.as_mut().unwrap() }
    }
}

impl UserStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

struct AppManager {
    num_app: usize,
    current_app: usize,
    app_start: [usize; MAX_APP_NUM + 1],
    app_names: [[usize; 2]; MAX_APP_NUM],
}

impl AppManager {
    fn get_app_name(&self, i: usize) -> &str {
        unsafe { str::from_utf8(core::slice::from_raw_parts(self.app_names[i][0] as *const u8, self.app_names[i][1])).expect("App name parse error") }
    }

    pub fn print_app_info(&self) {
        println!("[kernel] num_app = {}", self.num_app);
        for i in 0..self.num_app {
            println!(
                "[kernel] app_{} {} [{:#x}, {:#x})",
                i,
                self.get_app_name(i),
                self.app_start[i], 
                self.app_start[i + 1]
            );
        }
    }

    unsafe fn load_app(&self, app_id: usize) {
        if app_id >= self.num_app {
            panic!("All applications completed!");
        }
        println!("[kernel] Loading app_{} {}", app_id, self.get_app_name(app_id));
        // clear icache
        asm!("fence.i");
        // clear app area
        core::slice::from_raw_parts_mut(
            APP_BASE_ADDRESS as *mut u8,
            APP_SIZE_LIMIT
        ).fill(0);
        let app_src = core::slice::from_raw_parts(
            self.app_start[app_id] as *const u8,
            self.app_start[app_id + 1] - self.app_start[app_id]
        );
        let app_dst = core::slice::from_raw_parts_mut(
            APP_BASE_ADDRESS as *mut u8,
            app_src.len()
        );
        app_dst.copy_from_slice(app_src);
    }

    pub fn get_current_app(&self) -> usize { self.current_app }

    pub fn move_to_next_app(&mut self) {
        self.current_app += 1;
    }
}

lazy_static! {
    static ref APP_MANAGER: UPSafeCell<AppManager> = unsafe { UPSafeCell::new({
        extern "C" { fn _num_app(); fn _num_app_name(); }
        let num_app_ptr = _num_app as usize as *const usize;
        let num_app = num_app_ptr.read_volatile();
        let mut app_start: [usize; MAX_APP_NUM + 1] = [0; MAX_APP_NUM + 1];
        let app_start_raw: &[usize] =  core::slice::from_raw_parts(
            num_app_ptr.add(1), num_app + 1
        );
        app_start[..=num_app].copy_from_slice(app_start_raw);
        let num_name_ptr = _num_app_name as usize as *const usize;
        let mut app_names: [[usize; 2]; MAX_APP_NUM] = [[0; 2]; MAX_APP_NUM];
        let app_names_addr: &[usize] =
        core::slice::from_raw_parts(num_name_ptr, num_app);
        for i in 0..num_app {
            let ptr = app_names_addr[i] as *const usize;
            let t = ptr.read_volatile();
            app_names[i] = [app_names_addr[i] + 8, t];
        }
        AppManager {
            num_app,
            current_app: 0,
            app_start,
            app_names,
        }
    })};
}

pub fn init() {
    print_app_info();
}

pub fn print_app_info() {
    APP_MANAGER.exclusive_access().print_app_info();
}

static mut START_TIMER: usize = 0;

pub fn run_next_app() -> ! {
    let mut app_manager = APP_MANAGER.exclusive_access();
    let current_app = app_manager.get_current_app();
    unsafe {
        if current_app > 0 {
            let t = START_TIMER;
            START_TIMER = riscv::register::time::read();
            println!("[kernel] app_{0} executed in {1} ms", current_app - 1, (START_TIMER - t) / 10000);
        }
    }
    unsafe {
        app_manager.load_app(current_app);
    }
    app_manager.move_to_next_app();
    drop(app_manager);
    // before this we have to drop local variables related to resources manually
    // and release the resources
    extern "C" { fn __restore(cx_addr: usize); }
    unsafe {
        START_TIMER = riscv::register::time::read();
    }
    unsafe {
        __restore(KERNEL_STACK.push_context(
            TrapContext::app_init_context(APP_BASE_ADDRESS, USER_STACK.get_sp())
        ) as *const _ as usize);
    }
    panic!("Unreachable in batch::run_current_app!");
}

/// exercise 2.01, print current app's id & name
pub fn print_current_app_info() {
    let mut mana = APP_MANAGER.exclusive_access();
    let t = mana.current_app - 1;
    println!("Current app is app_{}, which name is {}", t, mana.get_app_name(t));
}