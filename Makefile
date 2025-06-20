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

# QEMU extra flags
QEMU_FLAGS ?=

# USTAR archive creation (required before kernel build)
.PHONY: ustar-archive
ustar-archive:
ifneq ($(USTAR),)
ifeq ($(USTAR),1)
	@echo "Creating archive.tar from ustar-files/ ..."
	tar --format=ustar -cvf archive.tar ustar-files/
endif
endif

.PHONY: run clean build-kernel build-bootloader check-artifacts esp fat iso qemu rust-clean

run: iso
	# Run with QEMU
	$(MAKE) qemu

build-bootloader: ustar-archive
	cargo build -p polished_bootloader --target x86_64-unknown-uefi --features uefi $(if $(filter release,$(BOOTLOADER_BUILD_DIR)),--release,)

build-kernel: ustar-archive
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
ifneq ($(USTAR),)
ifeq ($(USTAR),1)
	# archive.tar is created by ustar-archive target
	# Do not copy archive.tar to iso/ when USTAR=1
endif
endif
	xorriso -as mkisofs -R -f -o $(ISO_FILE) iso \
		-e $(FAT_IMG) -no-emul-boot

# QEMU targets
# Default: graphical QEMU
qemu: iso disk
	qemu-system-x86_64 \
		-drive if=pflash,format=raw,readonly=on,file=$(OVMF_CODE) \
		-drive format=raw,file=$(ISO_FILE) \
		-device virtio-blk-pci,drive=vdisk \
		-device pci-testdev \
		-drive id=vdisk,file=disk.img,if=none,format=raw \
		-smp 4 -m 6G -cpu max \
		-audiodev pa,id=snd0 -machine pcspk-audiodev=snd0 --serial stdio -M q35 --no-reboot \
		$(QEMU_FLAGS)

# Headless (no graphical output)
qemu-nographic: iso disk
	qemu-system-x86_64 \
		-drive if=pflash,format=raw,readonly=on,file=$(OVMF_CODE) \
		-drive format=raw,file=$(ISO_FILE) \
		-device virtio-blk-pci,drive=vdisk \
		-device pci-testdev \
		-drive id=vdisk,file=disk.img,if=none,format=raw \
		-smp 4 -m 6G -cpu max \
		-audiodev pa,id=snd0 -machine pcspk-audiodev=snd0 -M q35 --no-reboot \
		-nographic \
		$(QEMU_FLAGS)

# QEMU with GDB stub (graphical)
qemu-gdb: iso disk
	qemu-system-x86_64 \
		-drive if=pflash,format=raw,readonly=on,file=$(OVMF_CODE) \
		-drive format=raw,file=$(ISO_FILE) \
		-device virtio-blk-pci,drive=vdisk \
		-device pci-testdev \
		-drive id=vdisk,file=disk.img,if=none,format=raw \
		-smp 4 -m 6G -cpu max \
		-audiodev pa,id=snd0 -machine pcspk-audiodev=snd0 --serial stdio -M q35 --no-reboot \
		-s -S \
		-d unimp,guest_errors \
		$(QEMU_FLAGS)

# QEMU with GDB stub and no graphics
qemu-gdb-nographic: iso disk
	qemu-system-x86_64 \
		-drive if=pflash,format=raw,readonly=on,file=$(OVMF_CODE) \
		-drive format=raw,file=$(ISO_FILE) \
		-device virtio-blk-pci,drive=vdisk \
		-device pci-testdev \
		-drive id=vdisk,file=disk.img,if=none,format=raw \
		-smp 4 -m 6G -cpu max \
		-audiodev pa,id=snd0 -machine pcspk-audiodev=snd0 -M q35 --no-reboot \
		-nographic \
		-s -S \
		-d unimp,guest_errors \
		$(QEMU_FLAGS)

# QEMU with extra debug output (interrupts)
qemu-debug: iso disk
	qemu-system-x86_64 \
		-drive if=pflash,format=raw,readonly=on,file=$(OVMF_CODE) \
		-drive format=raw,file=$(ISO_FILE) \
		-device virtio-blk-pci,drive=vdisk \
		-device pci-testdev \
		-drive id=vdisk,file=disk.img,if=none,format=raw \
		-smp 4 -m 6G -cpu max \
		-audiodev pa,id=snd0 -machine pcspk-audiodev=snd0 -M q35 --no-reboot \
		-d int \
		$(QEMU_FLAGS)

qemu-debug-nographic: iso disk
	qemu-system-x86_64 \
		-drive if=pflash,format=raw,readonly=on,file=$(OVMF_CODE) \
		-drive format=raw,file=$(ISO_FILE) \
		-device virtio-blk-pci,drive=vdisk \
		-device pci-testdev \
		-drive id=vdisk,file=disk.img,if=none,format=raw \
		-smp 4 -m 6G -cpu max \
		-audiodev pa,id=snd0 -machine pcspk-audiodev=snd0 -M q35 --no-reboot \
		-nographic \
		-d int \
		$(QEMU_FLAGS)

# Create a virtio disk image
disk:
	qemu-img create -f raw disk.img 64M

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