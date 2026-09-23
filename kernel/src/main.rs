#![no_std]
#![no_main]

mod klib;
mod mm;
mod tty;

use crate::tty::printk::Arg;

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn kernel_start() -> ! {
    mm::gdt::init_gdt();

    printk!("%d! Hello, world!\n", Arg::Int(42));

    klib::dump_stack::dump_stack();
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
