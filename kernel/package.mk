PACKAGES += kernel

PHONY += pkg/kernel/install
pkg/kernel/install: $(SYSROOT)/boot/atom

$(SYSROOT)/boot/atom: $(PKGS)/kernel/atom | $(SYSROOT)/boot/
	@echo "  INSTALL  $@"
	rm -f $@
	cp $< $@

pkgKernelOut := $(PKGS)/kernel
pkgKernelOut := $(abspath $(pkgKernelOut))

$(PKGS)/kernel/atom: FORCE | $(PKGS)/kernel/
	@echo "  MAKE  pkg/kernel"
	$(MAKE) -C $(SRCTREE)/kernel OUT=$(pkgKernelOut) -f build.mk
