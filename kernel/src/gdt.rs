#[repr(C)]
#[allow(dead_code)]
pub struct GlobalDescriptorTable {
    // GDT의 엔트리들을 저장할 배열
    entries: [Descriptor; 7],
}

impl GlobalDescriptorTable {
    pub const fn new(entries: [Descriptor; 7]) -> Self {
        GlobalDescriptorTable { entries }
    }
}

#[repr(C, packed)]
#[allow(dead_code)]
pub struct Descriptor {
    // GDT 엔트리의 필드들을 정의
    limit_low: u16,  // 2바이트(0-15), limit의 하위 16비트
    base_low: u16,   // 2바이트(16-31), base의 하위 16비트
    base_middle: u8, // 1바이트(32-39), base의 중간 8비트(16-23)
    access: u8,      // 1바이트(40-47), access byte
    limit_flag: u8,  // 1바이트(48-55), limit의 상위 4비트와 flags
    base_high: u8,   // 1바이트(56-63), base의 상위 8비트
}

impl Descriptor {
    pub const fn new(base: u32, limit: u32, access: u8, flags: u8) -> Self {
        // limit: (20비트: 0 - 19)
        // 하위 16비트: 0xFFFF = 0b 1111 1111 1111 1111
        let limit_low: u16 = (limit & 0xFFFF) as u16;
        // 상위 4비트: 0x0F = 0b 0000 FFFF
        let limit_high: u8 = ((limit >> 16) & 0x0F) as u8;

        // base (32비트: 0 - 31)
        // 하위 16비트
        let base_low: u16 = (base & 0xFFFF) as u16;
        // 중간 8비트: 0xFF(0b1111_1111)로 하위 8비트만 남김
        let base_middle: u8 = ((base >> 16) & 0xFF) as u8;
        // 상위 8비트: 사실 24비트를 밀어버려서 마스크가 없어도 될듯 ...?
        let base_high: u8 = ((base >> 24) & 0xFF) as u8;

        // limit flag 바이트 조립
        // 상위 4비트 flag, 하위 4비트 limit
        // flag를 먼저 4비트로 밀어서 상위로 옯김
        // limit_high와 OR로 합침
        let limit_flag: u8 = (flags << 4) | limit_high;

        Descriptor {
            limit_low,
            base_low,
            base_middle,
            access,
            limit_flag,
            base_high,
        }
    }
}

#[repr(C, packed)]
struct Gdtr {
    limit: u16,
    base: u32,
}

// GDT 엔트리의 access byte에서 사용되는 플래그들을 정의
// zero-cost abstraction을 위해 상수로 정의
// zero-cost abstraction: 런타임에 오버헤드 없이 컴파일 타임에 상수로 치환되는 추상화
#[allow(dead_code)]
mod access {
    const PRESENT: u8 = 1 << 7; // 1: 세그먼트 존재, 0: 세그먼트 미존재 0x80

    const DPL_RING0: u8 = 0 << 5; // 0: 커널 모드 0x00
    const DPL_RING1: u8 = 1 << 5; // 1: 사용자 모드 0x20
    const DPL_RING2: u8 = 2 << 5; // 2: 사용자 모드 0x40
    const DPL_RING3: u8 = 3 << 5; // 3: 사용자 모드 0x60

    const CODE_DATA: u8 = 1 << 4; // 0: 시스템 세그먼트, 1: 코드/데이터 세그먼트 0x10
    const EXEC: u8 = 1 << 3; // 1: 실행 가능, 0: 실행 불가능 0x08
    const DC: u8 = 1 << 2; // 데이터(E=0) → Direction: 0=expand-up, 1=expand-down
                           //코드(E=1) → Conforming: 0=이 DPL에서만, 1=같거나 낮은 특권도 실행 가능
    const RW: u8 = 1 << 1; // 데이터 세그먼트 - 1: 읽기/쓰기 가능, 0: 읽기 전용
                           // 코드 세그먼트 - 1: 읽기 가능, 0: 읽기 불가능 0x02
    const ACCESSED: u8 = 1 << 0; // 1: 접근됨, 0: 접근 안됨 0x01 -> OS 단에서 초기화 시 항상 0으로 설정

    pub const KERNEL_CODE: u8 = PRESENT | DPL_RING0 | CODE_DATA | EXEC | RW; // 0x9A
    pub const KERNEL_DATA: u8 = PRESENT | DPL_RING0 | CODE_DATA | RW; // 0x92
    pub const KERNEL_STACK: u8 = PRESENT | DPL_RING0 | CODE_DATA | RW; // KERNEL_STACK == KERNEL_DATA

    // 데이터와 스택이 동일한 이유
    // x86에는 스택 전용 디스크립터 타입이 없다 — SS에 로드된 데이터 세그먼트가 곧 스택
    // DC(expand-down)를 켜지 말 것 — flat에서는 limit이 경계가 아니라 무의미하고, 켜면 유효 범위가 공집합이 되어 #GP
    // #GP : General Protection Fault, 일반 보호 오류
    // 여러가지 상황(특권 위반, 잘못된 셀렉터, 세그먼트 제한 초과 등) 발생하는 예외

    pub const USER_CODE: u8 = PRESENT | DPL_RING3 | CODE_DATA | EXEC | RW; // 0xFA
    pub const USER_DATA: u8 = PRESENT | DPL_RING3 | CODE_DATA | RW; // 0xF2
    pub const USER_STACK: u8 = PRESENT | DPL_RING3 | CODE_DATA | RW; // USER_STACK == USER_DATA
}

// GDT 엔트리의 limit flag 바이트에서 사용되는 플래그들을 정의
#[allow(dead_code)]
mod flags {
    const GRANULARITY: u8 = 1 << 3; // 1: 4KB 단위, 0: 바이트 단위 0x08
    const SIZE: u8 = 1 << 2; // 1: 32비트, 0: 16비트 0x04
    const LONG_MODE: u8 = 1 << 1; // 1: 64비트, 0: 32비트/16비트 0x02
    const AVL: u8 = 1 << 0; // x86 아키텍처에서 CPU가 사용하지 않는 비트, OS가 필요 시 자유롭게 사용

    pub const FLAT_32BIT: u8 = GRANULARITY | SIZE; // 0x0C
}

// 세그먼트 셀렉터 = GDT 인덱스 << 3 | TI | RPL
// TI(bit 2): 0이면 GDT, 1이면 LDT. RPL(bit 1-0): 요청 특권 레벨
// 인덱스는 init_gdt의 배열 순서와 반드시 일치해야 한다
#[allow(dead_code)]
mod selector {
    pub const KERNEL_CODE: u16 = 1 << 3; // 0x08
    pub const KERNEL_DATA: u16 = 2 << 3; // 0x10
    pub const KERNEL_STACK: u16 = 3 << 3; // 0x18

    // 링 3에 진입하지 않으므로 아직 로드하지 않는다. RPL을 3으로 둔다
    pub const USER_CODE: u16 = (4 << 3) | 3; // 0x23
    pub const USER_DATA: u16 = (5 << 3) | 3; // 0x2B
    pub const USER_STACK: u16 = (6 << 3) | 3; // 0x33
}

const _: () = {
    assert!(
        core::mem::size_of::<Descriptor>() == 8,
        "Descriptor size must be 8 bytes"
    );
    assert!(access::KERNEL_CODE == 0x9A, "KERNEL_CODE must be 0x9A");
    assert!(access::KERNEL_DATA == 0x92, "KERNEL_DATA must be 0x92");
    assert!(access::USER_CODE == 0xFA, "USER_CODE must be 0xFA");
    assert!(access::USER_DATA == 0xF2, "USER_DATA must be 0xF2");
    assert!(flags::FLAT_32BIT == 0x0C, "FLAT_32BIT must be 0x0C");
    assert!(access::KERNEL_STACK == access::KERNEL_DATA);
    assert!(access::USER_STACK == access::USER_DATA);
    assert!(
        core::mem::size_of::<GlobalDescriptorTable>() == 56,
        "GDT size must be 56 bytes"
    );
    // packed를 빠뜨리면 limit 뒤에 패딩 2바이트가 들어가 8바이트가 되고,
    // lgdt가 읽는 앞 6바이트에서 base가 어긋난다(0x800 -> 0x08000000)
    assert!(
        core::mem::size_of::<Gdtr>() == 6,
        "Gdtr must be 6 bytes (packed)"
    );
    assert!(selector::KERNEL_CODE == 0x08, "KERNEL_CODE selector must be 0x08");
    assert!(selector::KERNEL_DATA == 0x10, "KERNEL_DATA selector must be 0x10");
    assert!(selector::KERNEL_STACK == 0x18, "KERNEL_STACK selector must be 0x18");
};

pub fn init_gdt() {
    let gdt = GlobalDescriptorTable::new([
        Descriptor::new(0, 0, 0, 0), // Null descriptor
        Descriptor::new(0, 0xFFFFF, access::KERNEL_CODE, flags::FLAT_32BIT), // Kernel code
        Descriptor::new(0, 0xFFFFF, access::KERNEL_DATA, flags::FLAT_32BIT), // Kernel data
        Descriptor::new(0, 0xFFFFF, access::KERNEL_STACK, flags::FLAT_32BIT), // Kernel stack
        Descriptor::new(0, 0xFFFFF, access::USER_CODE, flags::FLAT_32BIT), // User code
        Descriptor::new(0, 0xFFFFF, access::USER_DATA, flags::FLAT_32BIT), // User data
        Descriptor::new(0, 0xFFFFF, access::USER_STACK, flags::FLAT_32BIT), // User stack
    ]);

    let gdtr = Gdtr {
        limit: (core::mem::size_of::<GlobalDescriptorTable>() - 1) as u16,
        base: 0x800,
    };

    unsafe {
        core::ptr::write(0x800 as *mut GlobalDescriptorTable, gdt);
        core::arch::asm!(
            "lgdt [{0}]",
            in(reg) &gdtr
        )
    };

    reload_segments();
}

// lgdt만으로는 새 GDT가 실제로 쓰이지 않는다.
// CPU는 세그먼트 레지스터에 셀렉터를 넣는 순간에만 디스크립터를 읽어
// 숨은 레지스터(hidden register)에 캐시하고, 이후에는 메모리를 다시 읽지 않는다.
// 따라서 전부 다시 로드해야 GRUB이 캐시해둔 디스크립터가 우리 것으로 교체된다.
//
// base가 0(flat)이므로 SS를 바꿔도 ESP는 그대로 유효하다.
fn reload_segments() {
    unsafe {
        // 세그먼트 레지스터에는 즉값을 직접 넣을 수 없어 ax를 경유한다.
        // ax를 건드리므로 out("eax") _ 로 클로버를 알려야 한다.
        core::arch::asm!(
            "mov ax, {data}",
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov ax, {stack}",
            "mov ss, ax",
            data = const selector::KERNEL_DATA,
            stack = const selector::KERNEL_STACK,
            out("eax") _,
        );

        // CS는 mov로 바꿀 수 없고 far jump만이 방법이다.
        // 바로 다음 라벨로 점프해 실행 흐름은 유지하면서 CS만 교체한다.
        // asm! 안에서는 이름 있는 라벨이 금지되므로 숫자 라벨(2: / $2f)을 쓴다.
        core::arch::asm!(
            "ljmp ${sel}, $2f",
            "2:",
            sel = const selector::KERNEL_CODE,
            options(att_syntax)
        );
    }
}
