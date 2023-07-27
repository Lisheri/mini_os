// src/main.rs 使用 linux系统的编写风格(LLVM默认风格)

#![no_std] // 不链接Rust标准库
#![no_main] // 禁用所有 rust 层级的入口点

use core::panic::PanicInfo;

// panic时调用如下函数
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle] // 不重整函数名
pub extern "C" fn _start() -> ! {
    // 编译器会寻找一个名为 '_start'的函数, 所以这个函数就是入口点
    // 默认命名为 '_start'
    loop {}
}