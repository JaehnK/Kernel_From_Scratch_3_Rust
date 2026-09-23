# 루트 Makefile — 빌드 순서: boot.o(as) → cargo(rust) → kernel.bin → 이미지 설치 → qemu
# nightly 전환은 rust-toolchain.toml이, -Z 플래그는 kernel/.cargo/config.toml이 담당

KERNEL_DIR  := kernel
TARGET      := i386-kernel
KERNEL_BIN  := kernel.bin
BUILD_BIN   := $(KERNEL_DIR)/target/$(TARGET)/release/kernel
IMG         := kfs.img
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

.PHONY: all build asm install run clean re

all: build

build: $(KERNEL_BIN)

asm: $(ASM_OBJ)

$(ASM_OBJ): $(ASM_SRC)
	$(AS) $(AS_FLAGS) $< -o $@

$(KERNEL_BIN): $(ASM_OBJ) $(RUST_SRC) $(BUILD_DEPS)
	cd $(KERNEL_DIR) && cargo build $(CARGO_FLAGS)
	cp $(BUILD_BIN) $@
	grub-file --is-x86-multiboot $@

# 이미지가 없을 때만 생성 — order-only(|)라서 커널이 갱신돼도 이미지를 다시 만들지 않는다
$(IMG):
	sh $(KERNEL_DIR)/scripts/init-image.sh $@

install: $(KERNEL_BIN) | $(IMG)
	@set -eu; \
	echo ">>> Mounting $(IMG)..."; \
	loop=$$(sudo losetup --show -fP $(IMG)); \
	sudo mkdir -p $(MOUNT_POINT); \
	sudo mount "$${loop}p1" $(MOUNT_POINT); \
	sudo cp $(KERNEL_BIN) $(MOUNT_POINT)/boot/kernel; \
	sudo umount $(MOUNT_POINT); \
	sudo losetup -d "$$loop"; \
	echo ">>> Kernel installed"

run: install
	qemu-system-i386 -drive file=$(IMG),format=raw -display curses \
		-monitor unix:/tmp/qemu-mon,server,nowait

clean:
	rm -f $(ASM_OBJ) $(KERNEL_BIN)
	cd $(KERNEL_DIR) && cargo clean

re: clean all
