const MULTIBOOT_MAGIC: u32 = 0x2BADB002;

// https://www.gnu.org/software/grub/manual/multiboot/multiboot.html
// 3.3 Boot Information Format 기반으로 작성
#[repr(C)]
pub struct MultibootInfo {
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

#[derive(Debug)]
#[expect(dead_code)]
pub enum BootError {
    InvalidMagic(u32), // 실제로 받은 magic 값
    NullInfo,          // info 주소가 0
}

pub struct MemInfo {
    pub lower: u32,
    pub upper: u32,
}

pub struct MmapInfo {
    pub length: u32,
    pub addr: u32,
}

const _: () = assert!(core::mem::offset_of!(MultibootInfo, mmap_addr) == 48);

impl MultibootInfo {
    /// Load the Multiboot Info from the given address.
    pub fn load(magic: u32, info: u32) -> Result<&'static MultibootInfo, BootError> {
        // 멀티부트 부팅을 위한 매직 시그널 확인
        if magic != MULTIBOOT_MAGIC {
            return Err(BootError::InvalidMagic(magic));
        }

        if info == 0 {
            return Err(BootError::NullInfo);
        }

        Ok(unsafe { &*(info as *const MultibootInfo) })
    }

    pub fn mmap_info(&self) -> Option<MmapInfo> {
        if self.flags & (1 << 6) == 0 {
            None
        } else {
            Some(MmapInfo {
                length: self.mmap_length,
                addr: self.mmap_addr,
            })
        }
    }

    pub fn mem_info(&self) -> Option<MemInfo> {
        if self.flags & (1 << 0) == 0 {
            None
        } else {
            Some(MemInfo {
                lower: self.mem_lower,
                upper: self.mem_upper,
            })
        }
    }
}
