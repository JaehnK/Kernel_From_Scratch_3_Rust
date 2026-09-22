use crate::vga;

fn utoa(mut n: u32, base: u32, buf: &mut [u8]) -> &[u8] {
    // b붙이는 이유: 바이트 문자열(c언어의 char *, ASCII)을 생성하기 위해
    let digits = b"0123456789abcdef";
    if base < 2 || base as usize > digits.len() {
        return &[];
    }
    // rust의 슬라이스의 신기방귀 똥방귀 포인트:
    // 1. 슬라이스 참조는 fat pointer(주소 + 길이)라서 길이 정보가 항상 같이 다님
    // 2. 스택 배열이든 힙이든, 슬라이스가 만들어지는 순간 길이가 확정되므로 이후엔 len()으로 안전하게 조회 가능
    // 3. 범위를 벗어난 인덱스 접근은 런타임 경계 검사에 걸려 panic으로 즉시 드러남
    // 4. 단, FFI로 받은 raw pointer에는 이 보장이 없음.
    //    from_raw_parts로 슬라이스를 만드는 지점에서 길이/유효성을 호출자가 서약해야 하며(unsafe),
    //    그 서약이 거짓이면 여기서의 안전도 무너짐

    let mut i = buf.len();
    loop {
        i -= 1;
        buf[i] = digits[(n % base) as usize];
        n /= base;
        if n == 0 {
            break;
        }
    }
    &buf[i..]
}

#[macro_export]
macro_rules! printk {
    ($fmt:expr $(, $arg:expr)* $(,)?) => {
        $crate::printk::vprintk($fmt, &[$(($arg),)*]);
    };
}

// 해당 printk에서 구현하고자 하는 포맷은
// %d(10진수), %x(16진수), %p(포인터), %s(문자열), %c(문자), %%로 제한한다.
pub enum Arg<'a> {
    // 'a: 변수 생애 주기 명시:
    Int(i32),
    Hex(u32),
    Char(u8),
    Str(&'a str),
}

pub fn vprintk(fmt: &str, args: &[Arg]) {
    let bytes = fmt.as_bytes(); // &str → &[u8], bytes[i]로 접근
    let mut arg_idx = 0;
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] != b'%' {
            vga::put_char(bytes[i]);
            i += 1;
            continue;
        } else {
            i += 1;
            if i as usize >= bytes.len() {
                break;
            }

            let specifier = bytes[i];
            match specifier {
                b'd' => {
                    if let Some(Arg::Int(v)) = args.get(arg_idx) {
                        let mut buf = [0u8; 32];
                        if *v < 0 {
                            vga::put_char(b'-');
                        }
                        let s = utoa(v.unsigned_abs(), 10, &mut buf);

                        vga::put_bytes(s);
                    }
                    arg_idx += 1;
                }
                b'x' => {
                    if let Some(Arg::Hex(v)) = args.get(arg_idx) {
                        let mut buf = [0u8; 32];
                        let s = utoa(*v, 16, &mut buf);
                        vga::put_bytes(s);
                    }
                    arg_idx += 1;
                }
                b'p' => {
                    if let Some(Arg::Hex(v)) = args.get(arg_idx) {
                        let mut buf = [0u8; 32];
                        let s = utoa(*v, 16, &mut buf);
                        for _ in 0..8usize.saturating_sub(s.len()) {
                            vga::put_char(b'0');
                        }
                        vga::put_bytes(s);
                    }
                    arg_idx += 1;
                }
                b's' => {
                    if let Some(Arg::Str(s)) = args.get(arg_idx) {
                        vga::put_str(s);
                    }
                    arg_idx += 1;
                }
                b'c' => {
                    if let Some(Arg::Char(v)) = args.get(arg_idx) {
                        vga::put_char(*v);
                    }
                    arg_idx += 1;
                }
                b'%' => vga::put_char(b'%'),
                _ => {}
            }
            i += 1;
        }
    }
}
