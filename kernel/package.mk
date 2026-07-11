PACKAGES += kernel

PHONY += pkg/kernel/install
pkg/kernel/install: $(SYSROOT)/boot/kernel

$(SYSROOT)/boot/kernel: $(PKGS)/kernel/kernel | $(SYSROOT)/boot/
	@echo "  INSTALL  $@"
	rm -f $@
	cp $< $@

pkgKernelOut := $(OUT)/packages/kernel
pkgKernelOut := $(abspath $(pkgKernelOut))

$(PKGS)/kernel/kernel: FORCE | $(PKGS)/kernel/
	@echo "  MAKE  pkg/kernel"
	$(MAKE) -C $(SRCTREE)/kernel OUT=$(pkgKernelOut) -f build.mk
