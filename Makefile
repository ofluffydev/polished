# Nuke built-in rules and variables for a clean slate
MAKEFLAGS += -rR
.SUFFIXES:

# User-overridable variables
KARCH ?= x86_64
QEMUFLAGS ?= -m 2G --serial stdio -no-reboot
# QEMUFLAGS ?= -m 2G -debugcon stdio -no-reboot
# QEMUFLAGS ?= -m 2G -debugcon stdio -no-reboot -d int
IMAGE_NAME := polished-$(KARCH)

# --- Kernel build logic (moved from kernel/Makefile) ---
# Target architecture to build for. Default to x86_64.
RUST_TARGET ?= $(KARCH)-unknown-none
ifeq ($(KARCH),riscv64)
    RUST_TARGET := riscv64gc-unknown-none-elf
endif
RUST_PROFILE ?= dev
RUST_PROFILE_SUBDIR := $(RUST_PROFILE)
ifeq ($(RUST_PROFILE),dev)
    RUST_PROFILE_SUBDIR := debug
endif
KERNEL_NAME := kernel
KERNEL_PATH := $(CURDIR)/target/$(RUST_TARGET)/$(RUST_PROFILE_SUBDIR)/$(KERNEL_NAME)

# Limine paths
LIMINE_DIR = limine
LIMINE_BINARIES = $(LIMINE_DIR)/limine-bios.sys $(LIMINE_DIR)/limine-bios-cd.bin $(LIMINE_DIR)/limine-uefi-cd.bin $(LIMINE_DIR)/BOOTX64.EFI $(LIMINE_DIR)/BOOTIA32.EFI

# OVMF firmware paths
OVMF_CODE = ovmf/ovmf-code-$(KARCH).fd
OVMF_VARS = ovmf/ovmf-vars-$(KARCH).fd

.PHONY: all
all: $(IMAGE_NAME).iso

.PHONY: run
run: run-x86_64

.PHONY: qemu
qemu: run-x86_64

.PHONY: run-x86_64
run-x86_64: $(OVMF_CODE) $(OVMF_VARS) $(IMAGE_NAME).iso
	qemu-system-$(KARCH) \
		-M q35 \
		-drive if=pflash,unit=0,format=raw,file=$(OVMF_CODE),readonly=on \
		-drive if=pflash,unit=1,format=raw,file=$(OVMF_VARS) \
		-cdrom $(IMAGE_NAME).iso \
		$(QEMUFLAGS)

# Limine build (clone if missing, build binaries)
$(LIMINE_DIR)/limine:
	rm -rf $(LIMINE_DIR)
	git clone https://github.com/limine-bootloader/limine.git --branch=v9.x-binary --depth=1
	$(MAKE) -C $(LIMINE_DIR)

# OVMF firmware download (x86_64 only)
$(OVMF_CODE):
	mkdir -p ovmf
	curl -Lo $@ https://github.com/osdev0/edk2-ovmf-nightly/releases/latest/download/ovmf-code-$(KARCH).fd

$(OVMF_VARS):
	mkdir -p ovmf
	curl -Lo $@ https://github.com/osdev0/edk2-ovmf-nightly/releases/latest/download/ovmf-vars-$(KARCH).fd

# --- Kernel build (now in main Makefile) ---
.PHONY: build-kernel
build-kernel:
	cd kernel && RUSTFLAGS="-C relocation-model=static" cargo build --target $(RUST_TARGET) --profile $(RUST_PROFILE)
	cp target/$(RUST_TARGET)/$(RUST_PROFILE_SUBDIR)/kernel kernel/kernel

# USTAR archive creation (preserved)
.PHONY: ustar-archive
ustar-archive:
ifneq ($(USTAR),)
ifeq ($(USTAR),1)
	@echo "Creating archive.tar from ustar-files/ ..."
	tar --format=ustar -cvf archive.tar ustar-files/
endif
endif

# ISO build using Limine (x86_64 only)
$(IMAGE_NAME).iso: $(LIMINE_DIR)/limine build-kernel
	rm -rf iso_root
	mkdir -p iso_root/boot/limine
	mkdir -p iso_root/EFI/BOOT
	# Always copy the freshly built kernel from the correct target directory
	cp -v kernel/kernel iso_root/boot/$(KERNEL_NAME)
	cp -v limine.conf iso_root/boot/limine/ || true
	cp -v $(LIMINE_DIR)/limine-bios.sys $(LIMINE_DIR)/limine-bios-cd.bin $(LIMINE_DIR)/limine-uefi-cd.bin iso_root/boot/limine/
	cp -v $(LIMINE_DIR)/BOOTX64.EFI iso_root/EFI/BOOT/
	cp -v $(LIMINE_DIR)/BOOTIA32.EFI iso_root/EFI/BOOT/
	xorriso -as mkisofs -b boot/limine/limine-bios-cd.bin \
		-no-emul-boot -boot-load-size 4 -boot-info-table \
		--efi-boot boot/limine/limine-uefi-cd.bin \
		-efi-boot-part --efi-boot-image --protective-msdos-label \
		iso_root -o $(IMAGE_NAME).iso
	./$(LIMINE_DIR)/limine bios-install $(IMAGE_NAME).iso
	rm -rf iso_root

# Clean targets (preserve kernel clean, remove old images)
.PHONY: clean
clean:
	cd kernel && cargo clean && rm -rf kernel
	rm -rf iso_root $(IMAGE_NAME).iso $(IMAGE_NAME).hdd

# --- Utility and publish targets (unchanged) ---
format:
	@echo "Formatting Rust code with cargo fmt..."
	cargo fmt --all
	@echo "Formatting TOML files with taplo..."
	taplo format '**/*.toml'
	@echo "Formatting Markdown files with mdformat..."
	mdformat .
	@echo "Formatting JSON files with jq..."
	for f in *.json; do jq . "$$f" > tmp.json && mv tmp.json "$$f"; done
	@echo "Done formatting files."

test:
	@echo "Running all tests..."
	clear && cargo test --workspace --exclude kernel --exclude polished_bootloader

publish:
	-cargo publish -p polished_bootloader     --allow-dirty --no-verify --target x86_64-unknown-uefi
	-cargo publish -p polished_files          --allow-dirty --no-verify
	-cargo publish -p polished_graphics       --allow-dirty --no-verify
	-cargo publish -p polished_panic_handler  --allow-dirty --no-verify
	-cargo publish -p polished_scancodes      --allow-dirty --no-verify
	-cargo publish -p polished_elf_loader     --allow-dirty --no-verify
	-cargo publish -p polished_gdt            --allow-dirty --no-verify
	-cargo publish -p polished_interrupts     --allow-dirty --no-verify
	-cargo publish -p polished_memory         --allow-dirty --no-verify
	-cargo publish -p polished_ps2            --allow-dirty --no-verify
	-cargo publish -p polished_serial_logging --allow-dirty --no-verify
	-cargo publish -p polished_x86_commands   --allow-dirty --no-verify
	-cargo publish -p polished_pci            --allow-dirty --no-verify
	-cargo publish -p polished_allocators     --allow-dirty --no-verify

publish-dry-run:
	-cargo publish -p polished_bootloader     --allow-dirty --dry-run --target x86_64-unknown-uefi
	-cargo publish -p polished_files          --allow-dirty --dry-run
	-cargo publish -p polished_graphics       --allow-dirty --dry-run
	-cargo publish -p polished_panic_handler  --allow-dirty --dry-run
	-cargo publish -p polished_scancodes      --allow-dirty --dry-run
	-cargo publish -p polished_elf_loader     --allow-dirty --dry-run
	-cargo publish -p polished_gdt            --allow-dirty --dry-run
	-cargo publish -p polished_interrupts     --allow-dirty --dry-run
	-cargo publish -p polished_memory         --allow-dirty --dry-run
	-cargo publish -p polished_ps2            --allow-dirty --dry-run
	-cargo publish -p polished_serial_logging --allow-dirty --dry-run
	-cargo publish -p polished_x86_commands   --allow-dirty --dry-run
	-cargo publish -p polished_pci            --allow-dirty --dry-run
	-cargo publish -p polished_allocators     --allow-dirty --dry-run

push:
	git push origin --all
	git push github --all
	git push origin --tags
	git push github --tags