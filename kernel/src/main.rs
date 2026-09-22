#![no_std]
#![no_main]

mod dump_stack;
mod gdt;
mod printk;
mod vga;

use crate::vga::*;

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn kernel_start() -> ! {
    gdt::init_gdt();

    printk::vprintk("%d! Hello, world!\n", &[42]);

    dump_stack::dump_stack();
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
