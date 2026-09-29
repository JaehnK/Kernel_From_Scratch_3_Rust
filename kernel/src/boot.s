.set MAGIC,    0x1BADB002
.set FLAGS,    (1 << 1)
.set CHECKSUM, -(MAGIC + FLAGS)

.section .text
.align 4
    .long MAGIC
    .long FLAGS
    .long CHECKSUM

.global _start
.type _start, @function
_start:
    mov $stack_top, %esp
    push %ebx # Multiboot 정보 구조체의 물리 주소, 두번째 인자(info)
    push %eax # 0x2BADB002 : 멀티부트의 매직 시그널 첫번째 인자(magic)
    call kernel_start
    cli
1:  hlt
    jmp 1b

.section .bss
.align 16
.global stack_bottom
.global stack_top
stack_bottom:
    .skip 16384
stack_top:
