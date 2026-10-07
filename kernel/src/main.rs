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

    // make run-panic으로 빌드했을 때만 컴파일된다
    #[cfg(feature = "panic-test")]
    klib::panic::panic_tester(info);

    // printk!("{}", 42);
    log!(LogLevel::Info, "{}, Hello, world!\n", 42);

    let _boot = klib::multiboot::MultibootInfo::load(magic, info).expect("multiboot boot info");

    loop {}
}
