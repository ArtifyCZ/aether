default: help
.SUFFIXES:            # Delete the default suffixes

include common.mk

$(BUILD):
	mkdir -p $(BUILD)


.PHONY: all

ifeq ($(ARCH),x86_64)
all:: $(BUILD)/kernel.$(ARCH).iso

else ifeq ($(ARCH),aarch64)
all:: $(BUILD)/kernel.$(ARCH).img

endif

INIT_MODULE__INIT_ELF := $(BUILD)/init.$(ARCH).elf

.PHONY: $(INIT_MODULE__INIT_ELF)

$(INIT_MODULE__INIT_ELF): ; bazel build //init:init --config $(ARCH)_user && rm -f "$@" && cp "$$(bazel cquery //init:init --config $(ARCH)_user --output=files | head -n 1)" "$@"

KERNEL_MODULE__KERNEL_ELF := $(BUILD)/kernel.$(ARCH).elf

.PHONY: $(KERNEL_MODULE__KERNEL_ELF)

$(KERNEL_MODULE__KERNEL_ELF): ; mkdir -p $(BUILD) && bazel build //kernel:kernel --config $(ARCH) && rm -f "$@" && cp "$$(bazel cquery //kernel:kernel --config $(ARCH) --output=files | head -n 1)" "$@"


MAKE_LIMINE := $(MAKE) -C $(BUILD)/limine
MAKE_LIMINE += CC="$(CC)"
MAKE_LIMINE += CFLAGS="-g -O2 -pipe"
MAKE_LIMINE += CPPFLAGS=""
MAKE_LIMINE += LDFLAGS=""
MAKE_LIMINE += LIBS=""


isofiles_dir := $(BUILD)/isofiles/$(ARCH)

.PHONY: $(BUILD)/kernel.x86_64.iso
$(BUILD)/kernel.x86_64.iso: $(KERNEL_MODULE__KERNEL_ELF) $(INIT_MODULE__INIT_ELF) $(BUILD)/limine/limine $(BUILD)
	rm -rf $(isofiles_dir) || true
	mkdir -p $(isofiles_dir)/boot/limine/
	cp -v limine.conf $(isofiles_dir)/boot/limine/
	mkdir -p $(isofiles_dir)/EFI/BOOT

	cp $(KERNEL_MODULE__KERNEL_ELF) $(isofiles_dir)/boot/kernel.elf
	cp $(INIT_MODULE__INIT_ELF) $(isofiles_dir)/boot/init.elf
	cp Mik_8x16.psf $(isofiles_dir)/boot/kernel-font.psf

	cp -v $(BUILD)/limine/limine-bios.sys $(BUILD)/limine/limine-bios-cd.bin $(BUILD)/limine/limine-uefi-cd.bin $(isofiles_dir)/boot/limine/
	cp -v $(BUILD)/limine/BOOTX64.EFI $(isofiles_dir)/EFI/BOOT/
	cp -v $(BUILD)/limine/BOOTIA32.EFI $(isofiles_dir)/EFI/BOOT/
	xorriso -as mkisofs -R -r -J -b boot/limine/limine-bios-cd.bin \
		-no-emul-boot -boot-load-size 4 -boot-info-table -hfsplus \
		-apm-block-size 2048 --efi-boot boot/limine/limine-uefi-cd.bin \
		-efi-boot-part --efi-boot-image --protective-msdos-label \
		$(isofiles_dir) -o $(BUILD)/kernel.$(ARCH).iso
	$(BUILD)/limine/limine bios-install $(BUILD)/kernel.$(ARCH).iso

.PHONY: $(BUILD)/kernel.aarch64.img
$(BUILD)/kernel.aarch64.img: $(KERNEL_MODULE__KERNEL_ELF) $(INIT_MODULE__INIT_ELF) $(BUILD)/limine/limine $(BUILD)
	(rm -rf $(BUILD)/kernel.aarch64.img || true)
	dd if=/dev/zero of=$@ bs=1M count=64
	mformat -i $@ ::
	mmd -i $@ ::/EFI ::/EFI/BOOT ::/boot ::/boot/limine
	mcopy -i $@ $(BUILD)/limine/BOOTAA64.EFI ::/EFI/BOOT/
	mcopy -i $@ $(srctree)/limine.conf ::/boot/limine/
	mcopy -i $@ $(KERNEL_MODULE__KERNEL_ELF) ::/boot/kernel.elf
	mcopy -i $@ $(INIT_MODULE__INIT_ELF) ::/boot/init.elf
	mcopy -i $@ $(srctree)/Mik_8x16.psf ::/boot/kernel-font.psf


$(BUILD)/limine/limine:
	rm -rf $(BUILD)/limine
	git clone https://github.com/limine-bootloader/limine.git $(BUILD)/limine --branch=v10.x-binary --depth=1
	$(MAKE_LIMINE)

$(BUILD)/raspi4b-uefi-firmware:
	(rm -rf $(BUILD)/raspi4b-uefi-firmware || true)
	mkdir -p $(BUILD)/raspi4b-uefi-firmware
	curl -L "https://github.com/pftf/RPi4/releases/download/v1.50/RPi4_UEFI_Firmware_v1.50.zip" \
		-o "$(BUILD)/raspi4b-uefi-firmware/firmware.zip"
	unzip $(BUILD)/raspi4b-uefi-firmware/firmware.zip -d $(BUILD)/raspi4b-uefi-firmware


QEMU := qemu-system-$(ARCH)

ifeq ($(ARCH),x86_64)
QEMU_IMAGE := $(BUILD)/kernel.$(ARCH).iso

QEMUFLAGS += -cdrom $(QEMU_IMAGE)
#QEMUFLAGS += -d int,cpu_reset
QEMUFLAGS += -serial stdio

else ifeq ($(ARCH),aarch64)
QEMU += -M virt,highmem=on,gic-version=2
QEMU_IMAGE := $(BUILD)/kernel.aarch64.img

QEMUFLAGS += -cpu cortex-a72 -m 2G
QEMUFLAGS += -bios $(QEMU_AARCH64_BIOS)
QEMUFLAGS += -drive file=$(QEMU_IMAGE),if=none,format=raw,id=hd0,readonly=on
QEMUFLAGS += -device virtio-blk-device,drive=hd0
QEMUFLAGS += -d int,mmu,guest_errors -D qemu.log
QEMUFLAGS += -device ramfb # TODO: make it work with virtio-gpu-pci and replace the ramfb
QEMUFLAGS += -device qemu-xhci -device usb-kbd
QEMUFLAGS += -chardev stdio,id=con0 -serial chardev:con0

else
$(error Architecture $(ARCH) not configured for Qemu)
endif


.PHONY: qemu qemu-debug

qemu: $(QEMU_IMAGE)
	$(QEMU) $(QEMUFLAGS)

qemu-debug: $(QEMU_IMAGE)
	$(QEMU) -s -S $(QEMUFLAGS)


## Removes all local artifacts
clean::
	rm -rf $(BUILD)/

.PHONY: help
## This help screen
help:
	@printf "Available targets:\n\n"
	@awk '/^[a-zA-Z\-_0-9%:\\]+/ { \
		helpMessage = match(lastLine, /^## (.*)/); \
		if (helpMessage) { \
		helpCommand = $$1; \
		helpMessage = substr(lastLine, RSTART + 3, RLENGTH); \
	gsub("\\\\", "", helpCommand); \
	gsub(":+$$", "", helpCommand); \
		printf "  \x1b[32;01m%-35s\x1b[0m %s\n", helpCommand, helpMessage; \
		} \
	} \
	{ lastLine = $$0 }' $(MAKEFILE_LIST) | sort -u
	@printf "\n"
