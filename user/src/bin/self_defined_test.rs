#![no_std]
#![no_main]

extern crate user_lib;

use user_lib::{fork, waitpid, eventfd, close, read, write, sleep, exit, println, EventFdFlags};

#[no_mangle]
fn main() -> i32 {
    let flags = [
        EventFdFlags::NONE,
        EventFdFlags::EFD_NONBLOCK,
        EventFdFlags::EFD_SEMAPHORE,
        EventFdFlags::EFD_NONBLOCK | EventFdFlags::EFD_SEMAPHORE,
    ];
    let mut fd = [0; 4];
    for i in 0..4 {
        fd[i] = eventfd(0, flags[i]) as usize;
    }
    println!("fd allocated");

    let mut buf = [0u8; 8];
    assert_eq!(read(fd[1], &mut buf), -2);
    assert_eq!(read(fd[3], &mut buf), -2);
    println!("failed read completed");

    let input = (1919810u64).to_ne_bytes();
    write(fd[1], &input);
    write(fd[3], &input);
    assert_eq!(read(fd[1], &mut buf), 0);
    assert_eq!(u64::from_ne_bytes(buf), 1919810u64);
    assert_eq!(read(fd[3], &mut buf), 1);
    println!("successful read completed");

    let pid = fork() as usize;
    if pid == 0 {
        sleep(500);
        println!("child wakes up");
        println!("child writes fd 0 & 2");
        write(fd[0], &input);
        write(fd[2], &input);
        exit(0);
    }

    println!("parent tries to read from fd 0 & 2 with blocking");
    assert_eq!(read(fd[0], &mut buf), 0);
    assert_eq!(u64::from_ne_bytes(buf), 1919810u64);
    assert_eq!(read(fd[2], &mut buf), 1);
    println!("parent read from fd 0 & 2 successfully");

    let mut exit_code = 0;
    waitpid(pid, &mut exit_code);
    println!("parent: child exited with code {}", exit_code);

    for i in 0..4 {
        close(fd[i]);
    }
    0
}
