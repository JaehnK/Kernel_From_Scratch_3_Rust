#![no_std]
#![no_main]

mod klib;
mod mm;
mod tty;

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "C" fn kernel_start(magic: u32, info: u32) -> ! {
    tty::init();
    mm::gdt::init_gdt();

    printk!("42 Hello, world!\n");
    let boot = klib::multiboot::MultibootInfo::load(magic, info).expect("multiboot boot info");
    klib::dump_stack::dump_stack();
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
