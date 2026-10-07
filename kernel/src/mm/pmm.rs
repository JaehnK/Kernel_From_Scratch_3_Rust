use core::ptr::addr_of;

unsafe extern "C" {
    static _kernel_start: u32;
    static _kernel_end: u32;
}

#[expect(dead_code, reason = "feat/pmm에서 사용 예정")]
pub fn kernel_range() -> (u32, u32) {
    let bottom = addr_of!(_kernel_start) as u32;
    let top = addr_of!(_kernel_end) as u32;
    (bottom, top)
}
