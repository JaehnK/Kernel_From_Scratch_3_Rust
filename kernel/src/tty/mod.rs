pub mod log;
pub mod printk;
pub mod vga;
pub fn init() {
    vga::clear();
}
