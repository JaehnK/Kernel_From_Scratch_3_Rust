use crate::printk;
use crate::tty::printk::Arg;
use core::ptr::addr_of;

// boot.s의 .bss에 예약된 커널 스택의 경계.
// 값을 읽는 것이 아니라 주소만 쓰므로 타입은 u8이면 충분하다.
unsafe extern "C" {
    static stack_bottom: u8;
    static stack_top: u8;
}

const BYTES_PER_ROW: u32 = 16;
// 화면이 80x25라 전체(최대 16KB)를 다 찍으면 스크롤로 다 흘러간다.
const MAX_ROWS: u32 = 16;

pub fn dump_stack() {
    let esp: u32;
    unsafe { core::arch::asm!("mov {}, esp", out(reg) esp) };

    let bottom = addr_of!(stack_bottom) as u32;
    let top = addr_of!(stack_top) as u32;

    // 스택은 아래로 자라므로 유효 범위는 [esp, top) 이다.
    // top은 경계 마커일 뿐 스택이 아니다(그 주소부터는 다른 .bss 변수).
    if esp < bottom || esp >= top {
        printk!(
            "stack: ESP %p is outside %p, %p\n",
            Arg::Hex(esp),
            Arg::Hex(bottom),
            Arg::Hex(top)
        );
        return;
    }

    let used = top - esp;
    let total = top - bottom;

    printk!("=== Kernel stack ===\n");
    printk!(
        " bottom %p   top %p\n",
        Arg::Hex(bottom),
        Arg::Hex(top)
    );
    printk!(
        " esp    %p   used %d / %d bytes\n",
        Arg::Hex(esp),
        Arg::Int(used as i32),
        Arg::Int(total as i32)
    );

    let mut addr = esp;
    let mut rows = 0;
    while addr < top && rows < MAX_ROWS {
        printk!("%p: ", Arg::Hex(addr));

        // 16진 컬럼(워드 단위). 마지막 행이 짧으면 공백으로 채워 ASCII 컬럼을 정렬한다.
        let mut w = 0;
        while w < 4 {
            let a = addr + w * 4;
            if a + 4 <= top {
                let v = unsafe { *(a as *const u32) };
                printk!("%p ", Arg::Hex(v));
            } else {
                printk!("         ");
            }
            w += 1;
        }

        // ASCII 컬럼(메모리 바이트 순서). 출력 불가 문자는 '.'으로 대체한다.
        printk!("|");
        let mut b = 0;
        while b < BYTES_PER_ROW {
            let a = addr + b;
            let c = if a < top {
                let raw = unsafe { *(a as *const u8) };
                if (0x20..0x7F).contains(&raw) {
                    raw
                } else {
                    b'.'
                }
            } else {
                b' '
            };
            printk!("%c", Arg::Char(c));
            b += 1;
        }
        printk!("|\n");

        addr += BYTES_PER_ROW;
        rows += 1;
    }

    if addr < top {
        printk!(
            " ... %d more bytes not shown\n",
            Arg::Int((top - addr) as i32)
        );
    }
}
