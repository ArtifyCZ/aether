PACKAGES += hello_world

PHONY += pkg/hello_world/install
pkg/hello_world/install: $(SYSROOT)/bin/hello_world

$(SYSROOT)/bin/hello_world: $(PKGS)/hello_world/hello_world | $(SYSROOT)/bin/
	@echo "  INSTALL  $@"
	rm -f $@
	cp $< $@

$(PKGS)/hello_world/hello_world: FORCE | $(PKGS)/hello_world/
	@echo "  BAZEL  //hello_world"
	cd $(SRCTREE) && bazel build //hello_world --config=$(ARCH)
	@rm -f $@
	cp $(SRCTREE)/bazel-bin/hello_world/hello_world $@
