pub mod gdt;
pub mod pmm;

pub const KERNEL_VIRT_BASE: u32 = 0xC000_0000;

pub const fn phys_to_virt(phys: u32) -> u32 {
    phys + KERNEL_VIRT_BASE
}
