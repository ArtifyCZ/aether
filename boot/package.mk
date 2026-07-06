PACKAGES += boot

pkgBootLimineCache := $(BUILD)/_cache/limine-v10.x-binary
pkgBootLimineArtifacts := \
    BOOTAA64.EFI BOOTIA32.EFI BOOTRISCV64.EFI BOOTX64.EFI \
    limine-bios-cd.bin limine-bios.sys limine-uefi-cd.bin

pkgBootSysrootFiles :=
pkgBootSysrootFiles += $(SYSROOT)/boot/limine/limine.conf
pkgBootSysrootFiles += $(SYSROOT)/boot/Mik_8x16.psf
pkgBootSysrootFiles += $(pkgBootLimineArtifacts:%=$(SYSROOT)/EFI/BOOT/%)
pkgBootSysrootFiles += $(pkgBootLimineArtifacts:%=$(SYSROOT)/boot/limine/%)

PHONY += pkg/boot/install
pkg/boot/install: $(pkgBootLimineCache) $(pkgBootSysrootFiles)

$(SYSROOT)/boot/limine/limine.conf: $(SRCTREE)/boot/limine/limine.conf | $(SYSROOT)/boot/limine/
	@echo "  INSTALL  $@"
	cp $< $@

$(SYSROOT)/boot/Mik_8x16.psf: $(SRCTREE)/boot/Mik_8x16.psf | $(SYSROOT)/boot/
	@echo "  INSTALL  $@"
	cp $< $@

$(SYSROOT)/EFI/BOOT/%: $(pkgBootLimineCache)/% | $(SYSROOT)/EFI/BOOT/ $(pkgBootLimine)
	@echo "  INSTALL  $@"
	cp $< $@

$(SYSROOT)/boot/limine/%: $(pkgBootLimineCache)/% | $(SYSROOT)/boot/limine/
	@echo "  INSTALL  $@"
	cp $< $@
