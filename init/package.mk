PACKAGES += init

PHONY += pkg/init/install
pkg/init/install: pkg/libs/libsyscall/install $(SYSROOT)/bin/init

$(SYSROOT)/bin/init: $(PKGS)/init/init | $(SYSROOT)/bin/
	@echo "  INSTALL  $@"
	rm -f $@
	cp $< $@

pkgInitOut := $(PKGS)/init
pkgInitOut := $(abspath $(pkgInitOut))

$(PKGS)/init/init: FORCE | $(PKGS)/init/
	@echo "  MAKE  pkg/init"
	$(MAKE) -C $(SRCTREE)/init OUT=$(pkgInitOut) -f build.mk
