use core::panic::PanicInfo;

use crate::klib::dump_stack;
use crate::log;
use crate::tty::log::LogLevel;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log!(LogLevel::Panic, "kernel panic: {}\n", info);
    dump_stack::dump_stack();

    unsafe {
        core::arch::asm!("cli");
    }
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

// 평가 시연용 패닉 시나리오. `make run-panic PANIC_TEST=<이름>`으로 고른다.
// 치명적 패닉은 한 번 나면 멈추므로 실행 한 번에 시나리오 하나만 가능하다.
#[cfg(feature = "panic-test")]
pub fn panic_tester(info: u32) -> ! {
    use crate::klib::multiboot::MultibootInfo;

    // 빌드할 때 환경 변수로 넘어온 값. 없으면 panic! 직접 호출
    let scenario = option_env!("PANIC_TEST").unwrap_or("panic");
    log!(LogLevel::Info, "panic test: {}\n", scenario);

    match scenario {
        // 언어가 자동으로 일으키는 패닉: 배열 범위 초과
        "oob" => {
            let arr = [1, 2, 3];
            // 상수 인덱스면 컴파일러가 미리 막으므로 black_box로 값을 숨긴다
            let i = core::hint::black_box(4242);
            log!(LogLevel::Info, "arr[{}] = {}\n", i, arr[i]);
        }
        // Option::unwrap 실패: 메시지 없이 위치만 나온다
        "unwrap" => {
            let none: Option<u32> = core::hint::black_box(None);
            none.unwrap();
        }
        // Result::expect 실패: 잘못된 Multiboot magic
        "magic" => {
            MultibootInfo::load(0x1234, info).expect("multiboot boot info");
        }

        "warn" => {
            let warned = crate::warn!(true, "non-fatal warning {}", 1);
            log!(
                LogLevel::Info,
                "still running after warning (returned {})\n",
                warned
            );
            panic!("fatal panic after warning");
        }

        _ => panic!("panic test: {}", 42),
    }
    unreachable!("panic test scenario did not panic");
}
