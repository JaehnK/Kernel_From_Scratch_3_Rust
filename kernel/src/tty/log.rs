use super::vga;
use core::fmt::{self, Write};

pub enum LogLevel {
    Info,
    // 시연 빌드(panic-test)에서는 warn!이 쓰이므로 일반 빌드일 때만 expect를 건다
    #[cfg_attr(
        not(feature = "panic-test"),
        expect(dead_code, reason = "일반 빌드에서는 warn!을 아직 쓰지 않음")
    )]
    Warn,
    Panic,
}

impl LogLevel {
    fn color(&self) -> u8 {
        match self {
            LogLevel::Info => 0x0f,
            LogLevel::Warn => 0x0e,
            LogLevel::Panic => 0x0c,
        }
    }
}

struct LogWriter {
    level: LogLevel,
}

impl Write for LogWriter {
    // 필수 메서드는 write_str 한개만 존재함
    // write_fmt는 Write 트레이트가 기본으로 제공하며, 포맷을 풀어 이 함수를 여러 번 호출한다.
    // write_str은 추상 함수처럼 시그니처만 있어 구현 필요
    fn write_str(&mut self, s: &str) -> fmt::Result {
        vga::put_str_attr(s, self.level.color());
        Ok(())
    }
}

#[macro_export]
macro_rules! log {
    ($level:expr, $($arg:tt)*) => {
        $crate::tty::log::vlog($level, format_args!($($arg)*))
    };
}

pub fn vlog(level: LogLevel, args: fmt::Arguments) {
    // VGA 쓰기는 실패하지 않음 - result를 버림(_ quite)
    let _ = LogWriter { level }.write_fmt(args);
}
