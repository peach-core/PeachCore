TARGET := riscv64gc-unknown-none-elf
MODE := release
APP_DIR := src/bin
TARGET_DIR := target/$(TARGET)/$(MODE)
APPS := $(wildcard $(APP_DIR)/*.rs)
ELFS := $(patsubst $(APP_DIR)/%.rs, $(TARGET_DIR)/%, $(APPS))
BINS := $(patsubst $(APP_DIR)/%.rs, $(TARGET_DIR)/%.bin, $(APPS))

OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64
CP := cp 

IMAGE_DIR := ../easy-fs-fuse/elf/

TEST ?= 

elf: $(APPS)
	@cargo build --release
ifeq ($(TEST), 1)
	@$(CP) $(TARGET_DIR)/usertests $(TARGET_DIR)/initproc
endif

binary: elf
	@$(foreach elf, $(ELFS), $(OBJCOPY) $(elf) --strip-all -O binary $(patsubst $(TARGET_DIR)/%, $(TARGET_DIR)/%.bin, $(elf));)

copy_to_elf_file: elf
	@mkdir -p ../easy-fs-fuse/elf/
	@$(foreach elf, $(ELFS), cp $(elf) $(patsubst $(TARGET_DIR)/%, $(IMAGE_DIR)/%, $(elf));)

build: binary copy_to_elf_file

clean:
	@cargo clean

.PHONY: elf binary build clean
