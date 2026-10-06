use core::fmt::{self, Write};

use super::vga;

// core::fmt가 만든 문자열 조각을 VGA로 보내는 연결부.
// 커서 등 상태는 vga.rs의 전역에 있으므로 필드가 필요 없다.
struct VgaWriter;

impl Write for VgaWriter {
    // 필수 메서드는 write_str 하나뿐이다.
    // write_fmt는 Write 트레이트가 기본으로 제공하며, 포맷을 풀어 이 함수를 여러 번 호출한다.
    // write_str은 추상 함수처럼 시그니처만 있어서 우리가 구현 필요
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

// fmt::Arguments는 리눅스 vprintk가 받는 va_list에 해당(포맷 + 인자 묶음).
pub fn vprintk(args: fmt::Arguments) {
    // VGA 쓰기는 실패하지 않으므로 Result를 버린다.
    let _ = VgaWriter.write_fmt(args);
}
