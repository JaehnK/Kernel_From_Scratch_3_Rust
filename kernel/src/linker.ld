/*
 * 커널 메모리 배치 (higher-half)
 *
 *   물리(LMA)            가상(VMA)
 *   0x00100000 .boot     0x00100000   ← 페이징 전에 실행: VMA = LMA
 *   0x001xxxxx .text~    0xC01xxxxx   ← 페이징 후에 실행: VMA = LMA + KERNEL_VIRT_BASE
 *
 * 부트로더는 PhysAddr(LMA) 기준으로 로드하고 진입점(_start)으로 점프한다.
 */

/* boot.s의 .set KERNEL_VIRT_BASE와 반드시 같은 값 */
KERNEL_VIRT_BASE = 0xC0000000;

/* _start는 .boot에 있으므로 진입점이 물리 주소가 된다 */
ENTRY(_start)

SECTIONS {
    . = 1M;

    /* .boot도 커널의 일부이므로 가상 주소 기준으로 시작점을 계산해 둔다 */
    _kernel_start = . + KERNEL_VIRT_BASE;

    /* AT() 없음 → VMA = LMA = 물리 1MB. 페이징을 켜기 전에 실행되는 코드 */
    .boot :
    {
        KEEP(*(.multiboot))  /* 헤더가 맨 앞 (파일 앞 8KB 안). 참조하는 곳이 없어 KEEP 필요 */
        *(.boot.text)        /* _start: 페이징 켜고 상위 주소로 점프 */
    }

    /* 여기부터 가상 주소. 아래 섹션은 AT()로 물리 주소에 로드된다 */
    . += KERNEL_VIRT_BASE;

    .text ALIGN (4K) : AT (ADDR (.text) - KERNEL_VIRT_BASE)
    {
        *(.text)
        *(.text.*)
    }

    .rodata ALIGN (4K) : AT (ADDR (.rodata) - KERNEL_VIRT_BASE)
    {
        *(.rodata)
        *(.rodata.*)
    }

    .data ALIGN (4K) : AT (ADDR (.data) - KERNEL_VIRT_BASE)
    {
        *(.data)
        *(.data.*)
    }

    .bss ALIGN (4K) : AT (ADDR (.bss) - KERNEL_VIRT_BASE)
    {
        *(.bss)
        *(.bss.*)
        *(COMMON)
    }

    /* 커널 스택(.bss)까지 포함한 끝. 가상 주소 */
    _kernel_end = .;

    /DISCARD/ :
    {
        *(.eh_frame)
        *(.debug_*)
        *(.note.*)
        *(.comment)
    }
}
