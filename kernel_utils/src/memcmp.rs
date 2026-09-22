#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcmp(ptr1: *const u8, ptr2: *const u8, n: usize) -> i32 {
    let mut idx: usize = 0;

    while idx < n {
        let byte1 = *ptr1.add(idx);
        let byte2 = *ptr2.add(idx);
        if byte1 != byte2 {
            return byte1 as i32 - byte2 as i32;
        }
        idx += 1;
    }
    0
}
