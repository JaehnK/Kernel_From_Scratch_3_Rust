use core::fmt::{self, Write};

use super::vga;
struct VgaWriter;

impl Write for VgaWriter {
    // 필수 메서드는 write_str 한개만 존재함
    // write_fmt는 Write 트레이트가 기본으로 제공하며, 포맷을 풀어 이 함수를 여러 번 호출한다.
    // write_str은 추상 함수처럼 시그니처만 있어 구현 필요
    fn write_str(&mut self, s: &str) -> fmt::Result {
        vga::put_str(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! printk {
    // 받은 토큰을 해석하지 않고 통째로 format_args!에 넘긴다.
    // 포맷 문자열과 인자의 개수·타입 검사는 format_args!가 컴파일 때 한다.
    ($($arg:tt)*) => {
        $crate::tty::printk::vprintk(format_args!($($arg)*))
    };
}

// fmt::Arguments는 리눅스 vprintk가 받는 va_list에 해당
pub fn vprintk(args: fmt::Arguments) {
    // VGA 쓰기는 실패하지 않음 - result를 버림(_ quite)
    let _ = VgaWriter.write_fmt(args);
}
