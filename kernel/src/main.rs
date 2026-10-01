#![no_std]
#![no_main]

mod klib;
mod mm;
mod tty;

use crate::tty::printk::Arg;
use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "C" fn kernel_start(magic: u32, info: u32) -> ! {
    tty::init();
    mm::gdt::init_gdt();

    let boot = klib::multiboot::MultibootInfo::load(magic, info).expect("multiboot boot info");
    printk!("%d! Hello, world!\n", Arg::Int(42));

    let mem_info = boot.mem_info().expect("Failed to get memory info");
    let mmap_info = boot.mmap_info().expect("Failed to get mmap info");
    printk!(
        "Memory Info: lower: %d KB, upper: %d KB\n",
        Arg::Int(mem_info.lower as i32),
        Arg::Int(mem_info.upper as i32)
    );
    printk!(
        "Mmap Info: length: %d, addr: 0x%p\n",
        Arg::Int(mmap_info.length as i32),
        Arg::Hex(mmap_info.addr)
    );
    klib::dump_stack::dump_stack();
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
