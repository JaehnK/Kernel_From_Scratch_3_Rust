# 루트 Makefile — 빌드 순서: boot.o(as) → cargo(rust) → kernel.bin → kfs.img
# make(all)은 GRUB 부팅 이미지까지 만들고(sudo), make run은 kernel.bin만으로 qemu를 띄운다(sudo 없음)
# nightly 전환은 rust-toolchain.toml이, -Z 플래그는 kernel/.cargo/config.toml이 담당

KERNEL_DIR  := kernel
TARGET      := i386-kernel
KERNEL_BIN  := kernel.bin
BUILD_BIN   := $(KERNEL_DIR)/target/$(TARGET)/release/kernel
IMG         := kfs.img
PANIC_BIN   := kernel-panic.bin
PANIC_TEST  ?= panic
MOUNT_POINT := /tmp/kfs_mount

ASM_SRC     := $(KERNEL_DIR)/src/boot.s
ASM_OBJ     := $(KERNEL_DIR)/src/boot.o

# 하위 디렉토리(memory/ 등)까지 재귀 탐색 — wildcard는 최상위 *.rs만 잡는다
RUST_SRC    := $(shell find $(KERNEL_DIR)/src kernel_utils/src -name '*.rs')
BUILD_DEPS  := $(KERNEL_DIR)/src/linker.ld $(KERNEL_DIR)/build.rs \
               $(KERNEL_DIR)/i386-kernel.json $(KERNEL_DIR)/Cargo.toml \
               $(KERNEL_DIR)/Cargo.lock $(KERNEL_DIR)/.cargo/config.toml \
               kernel_utils/Cargo.toml rust-toolchain.toml

AS          := as
AS_FLAGS    := --32
CARGO_FLAGS := --release
QEMU        := qemu-system-i386
QEMU_FLAGS  := -display curses -monitor unix:/tmp/qemu-mon,server,nowait

.PHONY: all build asm run run-img run-panic clean re

all: $(IMG)

build: $(KERNEL_BIN)

asm: $(ASM_OBJ)

$(ASM_OBJ): $(ASM_SRC)
	$(AS) $(AS_FLAGS) $< -o $@

$(KERNEL_BIN): $(ASM_OBJ) $(RUST_SRC) $(BUILD_DEPS)
	cd $(KERNEL_DIR) && cargo build $(CARGO_FLAGS)
	cp $(BUILD_BIN) $@
	grub-file --is-x86-multiboot $@

# 이미지가 없으면 생성하고, kernel.bin이 이미지보다 새로울 때만 커널을 교체한다
# 이미 최신이면 레시피가 돌지 않으므로 sudo도 묻지 않는다
$(IMG): $(KERNEL_BIN)
	@[ -e $@ ] || sh $(KERNEL_DIR)/scripts/init-image.sh $@
	@set -eu; \
	echo ">>> Mounting $@..."; \
	loop=$$(sudo losetup --show -fP $@); \
	sudo mkdir -p $(MOUNT_POINT); \
	sudo mount "$${loop}p1" $(MOUNT_POINT); \
	sudo cp $(KERNEL_BIN) $(MOUNT_POINT)/boot/kernel; \
	sudo umount $(MOUNT_POINT); \
	sudo losetup -d "$$loop"; \
	echo ">>> Kernel installed"
	@touch $@

# 개발용 — QEMU 내장 Multiboot 로더로 kernel.bin을 직접 부팅 (GRUB·이미지·sudo 불필요)
run: $(KERNEL_BIN)
	$(QEMU) -kernel $(KERNEL_BIN) $(QEMU_FLAGS)

# 제출물 확인용 — kfs.img의 GRUB으로 부팅 (Multiboot 정보가 run과 다를 수 있음)
run-img: $(IMG)
	$(QEMU) -drive file=$(IMG),format=raw $(QEMU_FLAGS)

# 평가 시연용 — 패닉 테스트를 켠 커널로 부팅. 시나리오: panic(기본) | oob | unwrap | magic
# 예: make run-panic PANIC_TEST=oob
# kernel.bin과 파일을 분리해 정상 빌드와 섞이지 않게 하고, 시나리오가 바뀔 수 있으므로 매번 cargo를 호출한다
run-panic: $(ASM_OBJ)
	cd $(KERNEL_DIR) && PANIC_TEST=$(PANIC_TEST) cargo build $(CARGO_FLAGS) --features panic-test
	cp $(BUILD_BIN) $(PANIC_BIN)
	$(QEMU) -kernel $(PANIC_BIN) $(QEMU_FLAGS)

clean:
	rm -f $(ASM_OBJ) $(KERNEL_BIN) $(PANIC_BIN)
	cd $(KERNEL_DIR) && cargo clean

re: clean all
