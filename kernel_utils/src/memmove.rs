#[unsafe(no_mangle)]
pub unsafe extern "C" fn memmove(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    if src < dest as *const u8 {
        for i in (0..n).rev() {
            unsafe { *dest.add(i) = *src.add(i) };
        }
    } else {
        for i in 0..n {
            unsafe { *dest.add(i) = *src.add(i) };
        }
    }
    dest
}
