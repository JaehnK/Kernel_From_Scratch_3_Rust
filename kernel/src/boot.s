# 멀티부트 헤더 상수
.set MAGIC,    0x1BADB002       # 부트로더가 헤더를 찾는 표시
.set FLAGS,    (1 << 1)         # bit 1: 메모리 정보(mem_, mmap_요청)
.set CHECKSUM, -(MAGIC + FLAGS) # 세 값을 더하면 0이 되도록 맞춤

# 부트로더 헤더
# 부트로더는 커널 앞 8kb안에서 해당 헤더를 검색함
# 링커 스크립트가 boot.o의 .text를 맨앞에 두므로 1MB에 위치함
.section .text
.align 4 # 헤더는 4바이트 정렬
    .long MAGIC
    .long FLAGS
    .long CHECKSUM

# 진입점
# 부트로더가 점프해서 오는 곳, 링커 스크립트에서 0x100000에 위치하도록 설정
# eax = magic, ebx = Multiboot 정보 구조체의 물리 주소
# 두 값은 kernel_start에 넘겨야 하므로, 페이징에서는 ecx에 저장해야 함
.global _start
.type _start, @function
_start:
    # 1: page_directory 물리 주소를 cr3에 설정
    # 제어 레지스터에서는 상수를 바로 넣을 수 없어 ecx 거침
    mov $boot_page_directory, %ecx
    mov %ecx, %cr3

    # 2: CR4 레지스터 설정
    # CR4.PSE(bit 4)를 켜서 PDE의 PS 비트(4MB 페이지)를 인식하게 함
    # 다른 비트는 유지해야 하므로 mov 대신 or 사용
    mov %cr4, %ecx # CR4 → ecx로 복사
    or $0x00000010, %ecx # PSE 비트 설정
    mov %ecx, %cr4 # 수정된 CR4 값을 다시 CR4에 저장

    # 3. CR0.PG(31번 비트)를 켜서 페이징 시작
    # CR0의 PE(bit 0, 보호 모드)가 꺼지지 않도록 역시 OR로 켠다
    # page directory와 PSE가 준비된 뒤에 켜야 하므로 마지막 단계
    mov %cr0, %ecx # CR0 값을 ecx로 복사
    or $0x80000000, %ecx # PG 비트 켜기
    mov %ecx, %cr0 # 수정된 CR0 값을 다시 CR0에 저장, 이 줄 이후로 페이징 시작

    # 4, 스택을 잡고 Rust 진입 함수 호출 (cdecl: 인자는 오른쪽부터 push)
    mov $stack_top, %esp # 스택은 아래로 자라므로 꼭대기에서 시작
    push %ebx # Multiboot 정보 구조체의 물리 주소, 두번째 인자(info)
    push %eax # 0x2BADB002 : 멀티부트의 매직 시그널 첫번째 인자(magic)
    call kernel_start # 돌아오지 않는 함수 (-> !)

    # kernel_start가 혹시 돌아오면 인터럽트를 끄고 영원히 정지
    cli # 일반 인터럽트 무시
1:  hlt # CPU 정지
    jmp 1b # NMI로 깨어나면 다시 hlt로


# 커널 스택(16KB)
# .bss는 파일에 내용이 없고 로드될 때 0으로 채워지는 영역
.section .bss
.align 16 # 스택 정렬
.global stack_bottom # dump_stack.rs에서 참조함
.global stack_top
stack_bottom:
    .skip 16384 # 16KB 스택 확보
stack_top:

# 부팅용 페이지 디렉토리 (4MB 페이지)
# 엔트리 하나가 가상 주소 4MB를 담당함(1024 * 4MB)
# 지금은 엔트리 0만 써서 가상 0~4MB → 물리 0~4MB(identity)로 매핑
.section .data
.align 4096 # CR3가 가르키는 주소는 4KB 정렬
boot_page_directory:
    .long 0x00000083 # 엔트리 0: 0000 0000 1000 0011 (P | RW | PS)
    .fill 1023, 4, 0 # 엔트리 1 - 1023: 나머지 페이지 테이블 항목은 모두 0으로 초기화
