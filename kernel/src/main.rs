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
    printk!(
        "Mem: %p, %p\n",
        Arg::Hex(boot.mem_info().unwrap().lower),
        Arg::Hex(boot.mem_info().unwrap().upper)
    );
    printk!(
        "Mmap: %p, %p\n",
        Arg::Hex(boot.mmap_info().unwrap().addr),
        Arg::Hex(boot.mmap_info().unwrap().length)
    );

    klib::dump_stack::dump_stack();
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
