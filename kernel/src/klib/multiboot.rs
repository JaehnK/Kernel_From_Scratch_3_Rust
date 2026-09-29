use crate::printk;
use crate::tty::printk::Arg;

// https://www.gnu.org/software/grub/manual/multiboot/multiboot.html
// 3.3 Boot Information Format 기반으로 작성
#[repr(C)]
struct MultibootInfo {
    flags: u32,
    mem_lower: u32,
    mem_upper: u32,
    boot_device: u32,
    cmdline: u32,
    mods_count: u32,
    mods_addr: u32,
    syms: [u32; 4],
    mmap_length: u32,
    mmap_addr: u32,
}

const _: () = assert!(core::mem::offset_of!(MultibootInfo, mmap_addr) == 48);

pub fn parse_multiboot_info(info: u32) {
    let mbi: &MultibootInfo = unsafe { &*(info as *const MultibootInfo) };

    printk!("Multiboot Info:\n");
    printk!("  flags: %x\n", Arg::Hex(mbi.flags));
    printk!("  mem_lower: %x KB\n", Arg::Hex(mbi.mem_lower));
    printk!("  mem_upper: %x KB\n", Arg::Hex(mbi.mem_upper));
    printk!("  boot_device: %x\n", Arg::Hex(mbi.boot_device));
    printk!("  cmdline: %x\n", Arg::Hex(mbi.cmdline));
    printk!("  mods_count: %d\n", Arg::Int(mbi.mods_count as i32));
    printk!("  mods_addr: %x\n", Arg::Hex(mbi.mods_addr));
    printk!(
        "  syms: [%x, %x, %x, %x]\n",
        Arg::Hex(mbi.syms[0]),
        Arg::Hex(mbi.syms[1]),
        Arg::Hex(mbi.syms[2]),
        Arg::Hex(mbi.syms[3])
    );
    printk!("  mmap_length: %d\n", Arg::Int(mbi.mmap_length as i32));
    printk!("  mmap_addr: %x\n", Arg::Hex(mbi.mmap_addr));
}
