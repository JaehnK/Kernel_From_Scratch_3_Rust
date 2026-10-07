#![no_std]
#![no_main]

mod klib;
mod mm;
mod tty;

use crate::tty::log::LogLevel;

#[unsafe(no_mangle)]
pub extern "C" fn kernel_start(magic: u32, info: u32) -> ! {
    tty::init();
    mm::gdt::init_gdt();

    // make run-panic으로 빌드했을 때 컴파일 PANIC_TEST=<magic, ...>
    #[cfg(feature = "panic-test")]
    klib::panic::panic_tester(info);

    // printk!("{}", 42);
    log!(LogLevel::Info, "{}, Hello, world!\n", 42);

    let _boot = klib::multiboot::MultibootInfo::load(magic, info)
        .unwrap_or_else(|e| panic!("multiboot: {}", e));

    loop {}
}
