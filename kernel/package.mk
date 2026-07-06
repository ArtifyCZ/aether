PACKAGES += kernel

PHONY += pkg/kernel/install
pkg/kernel/install: $(SYSROOT)/boot/kernel

$(SYSROOT)/boot/kernel: $(PKGS)/kernel/kernel | $(SYSROOT)/boot/
	@echo "  INSTALL  $@"
	rm -f $@
	cp $< $@

$(PKGS)/kernel/kernel: FORCE | $(PKGS)/kernel/
	@echo "  BAZEL  //kernel"
	cd $(SRCTREE) && bazel build //kernel --config=$(ARCH)
	@rm -f $@
	cp $(SRCTREE)/bazel-bin/kernel/entry/kernel $@
