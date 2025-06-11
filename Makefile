# Variables
OVMF_CODE = /usr/share/edk2/x64/OVMF_CODE.4m.fd
OVMF_VARS = /usr/share/edk2/x64/OVMF_VARS.4m.fd
# KERNEL_NAME = kernel
FAT_IMG = fat.img
ISO_FILE = polished.iso
# KERNEL_PATH = $(CURDIR)/kernel/target/x86_64-custom/release/$(KERNEL_NAME)
# BOOTLOADER_BUILD_DIR := $(if $(RELEASE),release,debug)
BOOTLOADER_BUILD_DIR := $(if $(RELEASE),release,debug)
BOOTLOADER_PATH = $(CURDIR)/target/x86_64-unknown-uefi/$(BOOTLOADER_BUILD_DIR)/polished_bootloader.efi
ESP_DIR = esp/efi/boot

# Kernel path variables
KERNEL_BUILD_DIR := $(if $(RELEASE),release,debug)
KERNEL_NAME = kernel
KERNEL_PATH = $(CURDIR)/target/x86_64-polished-kernel/$(KERNEL_BUILD_DIR)/$(KERNEL_NAME)

.PHONY: run clean build-kernel build-bootloader check-artifacts esp fat iso qemu rust-clean

run: iso
	# Run with QEMU
	$(MAKE) qemu

build-bootloader:
	cargo build -p polished_bootloader --target x86_64-unknown-uefi $(if $(filter release,$(BOOTLOADER_BUILD_DIR)),--release,)

build-kernel:
	env RUSTFLAGS="-C relocation-model=static -C link-args=-no-pie" \
	cargo build -p kernel -Zbuild-std=core,alloc --target x86_64-polished-kernel.json $(if $(filter release,$(KERNEL_BUILD_DIR)),--release,)

check-artifacts: build-kernel build-bootloader
	@if [ ! -f $(BOOTLOADER_PATH) ]; then echo "Error: bootloader.efi not found!"; exit 1; fi

esp: check-artifacts
	mkdir -p $(ESP_DIR)
	cp $(BOOTLOADER_PATH) $(ESP_DIR)/bootx64.efi
	cp $(KERNEL_PATH) $(ESP_DIR)/$(KERNEL_NAME)

fat: esp
	dd if=/dev/zero of=$(FAT_IMG) bs=1M count=33
	mformat -i $(FAT_IMG) -F ::
	mmd -i $(FAT_IMG) ::/EFI
	mmd -i $(FAT_IMG) ::/EFI/BOOT
	mcopy -i $(FAT_IMG) $(ESP_DIR)/bootx64.efi ::/EFI/BOOT
	mcopy -i $(FAT_IMG) $(ESP_DIR)/$(KERNEL_NAME) ::/EFI/BOOT

iso: fat
	mkdir -p iso
	cp $(FAT_IMG) iso/
	xorriso -as mkisofs -R -f -e $(FAT_IMG) -no-emul-boot -o $(ISO_FILE) iso

# QEMU targets
# Default: graphical QEMU
qemu: iso
	qemu-system-x86_64 \
		-drive if=pflash,format=raw,readonly=on,file=$(OVMF_CODE) \
		-drive format=raw,file=$(ISO_FILE) \
		-smp 4 -m 6G -cpu max \
		-audiodev pa,id=snd0 -machine pcspk-audiodev=snd0 --serial stdio -M q35 --no-reboot

# Headless (no graphical output)
qemu-nographic: iso
	qemu-system-x86_64 \
		-drive if=pflash,format=raw,readonly=on,file=$(OVMF_CODE) \
		-drive format=raw,file=$(ISO_FILE) \
		-smp 4 -m 6G -cpu max \
		-audiodev pa,id=snd0 -machine pcspk-audiodev=snd0 -M q35 --no-reboot \
		-nographic

# QEMU with GDB stub (graphical)
qemu-gdb: iso
	qemu-system-x86_64 \
		-drive if=pflash,format=raw,readonly=on,file=$(OVMF_CODE) \
		-drive format=raw,file=$(ISO_FILE) \
		-smp 4 -m 6G -cpu max \
		-audiodev pa,id=snd0 -machine pcspk-audiodev=snd0 --serial stdio -M q35 --no-reboot \
		-s -S \
		-d unimp,guest_errors

# QEMU with GDB stub and no graphics
qemu-gdb-nographic: iso
	qemu-system-x86_64 \
		-drive if=pflash,format=raw,readonly=on,file=$(OVMF_CODE) \
		-drive format=raw,file=$(ISO_FILE) \
		-smp 4 -m 6G -cpu max \
		-audiodev pa,id=snd0 -machine pcspk-audiodev=snd0 -M q35 --no-reboot \
		-nographic \
		-s -S \
		-d unimp,guest_errors

# QEMU with extra debug output (interrupts)
qemu-debug: iso
	qemu-system-x86_64 \
		-drive if=pflash,format=raw,readonly=on,file=$(OVMF_CODE) \
		-drive format=raw,file=$(ISO_FILE) \
		-smp 4 -m 6G -cpu max \
		-audiodev pa,id=snd0 -machine pcspk-audiodev=snd0 -M q35 --no-reboot \
		-d int

qemu-debug-nographic: iso
	qemu-system-x86_64 \
		-drive if=pflash,format=raw,readonly=on,file=$(OVMF_CODE) \
		-drive format=raw,file=$(ISO_FILE) \
		-smp 4 -m 6G -cpu max \
		-audiodev pa,id=snd0 -machine pcspk-audiodev=snd0 -M q35 --no-reboot \
		-nographic \
		-d int

rust-clean:
	cd kernel && cargo clean
	cd bootloader && cargo clean

clean: rust-clean
	rm -rf esp $(FAT_IMG) iso $(ISO_FILE)

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


publish:
	-cargo publish -p polished_bootloader --allow-dirty
	-cargo publish -p polished_files --allow-dirty
	-cargo publish -p polished_graphics --allow-dirty
	-cargo publish -p polished_panic_handler --allow-dirty
	-cargo publish -p polished_scancodes --allow-dirty
	-cargo publish -p polished_elf_loader --allow-dirty
	-cargo publish -p polished_gdt --allow-dirty
	-cargo publish -p polished_interrupts --allow-dirty
	-cargo publish -p polished_memory --allow-dirty
	-cargo publish -p polished_ps2 --allow-dirty
	-cargo publish -p polished_serial_logging --allow-dirty
	-cargo publish -p polished_x86_commands --allow-dirty