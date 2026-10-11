# 멀티부트 헤더 상수
.set MAGIC,    0x1BADB002       # 부트로더가 헤더를 찾는 표시
.set FLAGS,    (1 << 1)         # bit 1: 메모리 정보(mem_, mmap_) 요청
.set CHECKSUM, -(MAGIC + FLAGS) # 세 값을 더하면 0이 되도록 맞춤

# 커널의 가상 주소 시작점 (3G/1G 분할). linker.ld의 같은 이름과 반드시 같은 값
.set KERNEL_VIRT_BASE, 0xC0000000

# 멀티부트 헤더
# 부트로더는 커널 파일 앞 8KB 안에서 이 헤더를 찾는다.
# linker.ld가 .multiboot를 .boot 섹션 맨 앞(물리 1MB)에 둔다.
# 새 이름의 섹션은 기본이 비할당이므로 "a"(allocatable)로 메모리에 올리게 한다.
.section .multiboot, "a"
.align 4 # 헤더는 4바이트 정렬
    .long MAGIC
    .long FLAGS
    .long CHECKSUM

# 진입점 (물리 주소에서 실행)
# .boot.text는 VMA = LMA = 물리 1MB로 링크된다. 페이징을 켜기 전이므로
# 이 구간에서 다른 섹션(.data, .text 등)의 라벨은 가상 주소라 그대로 쓸 수 없다.
# eax = magic, ebx = Multiboot 정보 구조체의 물리 주소.
# 두 값은 kernel_start에 넘겨야 하므로 페이징 설정에는 ecx만 쓴다.
.section .boot.text, "ax"
.global _start
.type _start, @function
_start:
    # 1: page directory의 물리 주소를 CR3에 설정
    # boot_page_directory는 .data(가상 주소)에 있으므로 KERNEL_VIRT_BASE를 빼서 물리로 바꾼다.
    # 제어 레지스터에는 상수를 바로 넣을 수 없어 ecx를 거친다.
    mov $boot_page_directory - KERNEL_VIRT_BASE, %ecx
    mov %ecx, %cr3

    # 2: CR4.PSE(bit 4)를 켜서 PDE의 PS 비트(4MB 페이지)를 인식하게 함
    # 다른 비트는 유지해야 하므로 읽고 → OR → 다시 쓴다.
    mov %cr4, %ecx       # CR4 → ecx로 복사
    or $0x00000010, %ecx # PSE 비트 켜기
    mov %ecx, %cr4       # 수정된 값을 CR4에 저장

    # 3: CR0.PG(bit 31)를 켜서 페이징 시작
    # CR0의 PE(bit 0, 보호 모드)가 꺼지지 않도록 역시 OR로 켠다.
    # page directory와 PSE가 준비된 뒤에 켜야 하므로 마지막 단계.
    mov %cr0, %ecx       # CR0 → ecx로 복사
    or $0x80000000, %ecx # PG 비트 켜기
    mov %ecx, %cr0       # 이 다음 명령부터 주소 변환 시작 (PD[0] identity 덕분에 계속 실행됨)

    # 4: 상위 주소(0xC01xxxxx)로 절대 점프
    # `jmp higher_half`는 상대 점프라 물리 주소에서 거리만큼만 이동하므로,
    # 주소 값을 레지스터에 넣고 그 주소로 점프한다. PD[768] 덕분에 유효한 주소다.
    lea higher_half, %ecx
    jmp *%ecx

# 여기부터 가상 주소(0xC01xxxxx)에서 실행
.section .text
higher_half:
    # 5: 스택을 잡고 Rust 진입 함수 호출 (cdecl: 인자는 오른쪽부터 push)
    mov $stack_top, %esp # 스택은 아래로 자라므로 꼭대기에서 시작 (가상 주소)
    push %ebx            # 두 번째 인자: info (Multiboot 정보 구조체의 물리 주소)
    push %eax            # 첫 번째 인자: magic (0x2BADB002)
    call kernel_start    # 돌아오지 않는 함수 (-> !)

    # kernel_start가 혹시 돌아오면 인터럽트를 끄고 영원히 정지
    cli    # 일반 인터럽트 무시
1:  hlt    # CPU 정지
    jmp 1b # NMI로 깨어나면 다시 hlt로

# 커널 스택 (16KB)
# .bss는 파일에 내용이 없고 로드될 때 0으로 채워지는 영역
.section .bss
.align 16             # 스택 정렬
.global stack_bottom  # dump_stack.rs에서 참조
.global stack_top
stack_bottom:
    .skip 16384       # 16KB 스택 확보
stack_top:

# 부팅용 page directory (4MB 페이지)
# 엔트리 하나가 가상 주소 4MB를 담당한다 (1024 × 4MB = 4GB).
# PD[0]:   가상 0x00000000~ → 물리 0~4MB (identity). 페이징을 켠 직후 낮은 주소의 _start를 계속 실행하기 위함
# PD[768]: 가상 0xC0000000~ → 물리 0~4MB. 768 = 0xC0000000 >> 22. 커널 본체가 실행되는 곳
# 제대로 된 4KB 페이지 테이블은 feat/paging에서 Rust로 만들어 교체한다.
.section .data
.align 4096 # CR3가 가리키는 주소는 4KB 정렬
boot_page_directory:
    .long 0x00000083 # PD[0]:   물리 0번지 | PS(bit 7) | RW(bit 1) | P(bit 0)
    .fill 767, 4, 0  # PD[1 - 767]: 매핑 안 함
    .long 0x00000083 # PD[768]: PD[0]과 같은 물리 4MB를 가리킴
    .fill 255, 4, 0  # PD[769 - 1023]: 매핑 안 함
