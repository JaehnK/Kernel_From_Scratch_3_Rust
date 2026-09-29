#![no_std]
#![no_main]

mod klib;
mod mm;
mod tty;

use crate::tty::printk::Arg;

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "C" fn kernel_start(magic: u32, info: u32) -> ! {
    mm::gdt::init_gdt();

    // 멀티부트 부팅을 위한 매직 시그널 확인
    if magic != 0x2BADB002 {
        panic!("Invalid magic number: {:#x}", magic);
    }

    klib::multiboot::parse_multiboot_info(info);
    printk!("%d! Hello, world!\n", Arg::Int(42));

    klib::dump_stack::dump_stack();
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
