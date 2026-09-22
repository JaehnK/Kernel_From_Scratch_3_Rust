#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, i: usize) -> *mut u8 {
    let mut idx: usize = 0;

    while idx < i {
        *dest.add(idx) = *src.add(idx);
        idx += 1;
    }
    dest
}
