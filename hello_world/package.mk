PACKAGES += hello_world

PHONY += pkg/hello_world/install
pkg/hello_world/install: $(SYSROOT)/bin/hello_world

$(SYSROOT)/bin/hello_world: $(PKGS)/hello_world/hello_world | $(SYSROOT)/bin/
	@echo "  INSTALL  $@"
	rm -f $@
	cp $< $@

pkgHelloWorldOut := $(PKGS)/hello_world
pkgHelloWorldOut := $(abspath $(pkgHelloWorldOut))

$(PKGS)/hello_world/hello_world: FORCE | $(PKGS)/hello_world/
	@echo "  MAKE  pkg/hello_world"
	$(MAKE) -C $(SRCTREE)/hello_world OUT=$(pkgHelloWorldOut) -f build.mk
