use kernel_utils::memmove;

const VGA_BUFFER: *mut u16 = 0xb8000 as *mut u16; // u16으로 하는 이유는 2바이트로 구성된 문자와 속성을 함께 저장하기 위해서, 즉, 포인터 최소 단위와 글자 단위를 맞추기 위함
const CHAR_ATTR: u8 = 0x0f;

struct Cursor {
    col: usize,
    row: usize,
}
static mut CURSOR: Cursor = Cursor { col: 0, row: 0 };

pub fn put_str(s: &str) {
    for b in s.bytes() {
        put_char(b);
    }
}

pub fn put_char(c: u8) {
    match c {
        b'\n' => newline(),
        b'\r' => unsafe {
            CURSOR.col = 0;
        }, // carriage return
        _ => put_glyph(c),
    }
}

fn put_glyph(c: u8) {
    let glyph = (CHAR_ATTR as u16) << 8 | c as u16;
    unsafe {
        VGA_BUFFER
            .add(CURSOR.row * 80 + CURSOR.col)
            .write_volatile(glyph); // 컴파일러 단에서 필요없는 메모리라 판단하지 못하도록 작성을 강제함
    }
    advance();
}

fn advance() {
    unsafe {
        CURSOR.col += 1;
        if CURSOR.col >= 80 {
            newline();
        }
    }
}

fn newline() {
    unsafe {
        CURSOR.col = 0;
        CURSOR.row += 1;
        if CURSOR.row >= 25 {
            scroll();
            CURSOR.row = 24;
        }
    }
}

fn scroll() {
    unsafe {
        // 1~24행을 0~23행으로 (2000-80=1920셀, 셀당 2바이트)
        memmove(
            VGA_BUFFER as *mut u8,
            VGA_BUFFER.add(80) as *const u8,
            24 * 80 * 2,
        );

        // 마지막 줄을 빈 셀로 채우기
        let blank = (CHAR_ATTR as u16) << 8 | b' ' as u16;
        for i in 0..80 {
            VGA_BUFFER.add(24 * 80 + i).write_volatile(blank);
        }
    }
}

pub fn clear() {
    unsafe {
        let blank = (CHAR_ATTR as u16) << 8 | b' ' as u16;
        for i in 0..(25 * 80) {
            VGA_BUFFER.add(i).write_volatile(blank);
        }
        CURSOR.col = 0;
        CURSOR.row = 0;
    }
}
