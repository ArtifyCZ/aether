PACKAGES += init

PHONY += pkg/init/install
pkg/init/install: $(SYSROOT)/bin/init

$(SYSROOT)/bin/init: $(PKGS)/init/init | $(SYSROOT)/bin/
	@echo "  INSTALL  $@"
	rm -f $@
	cp $< $@

$(PKGS)/init/init: FORCE | $(PKGS)/init/
	@echo "  BAZEL  //init"
	cd $(SRCTREE) && bazel build //init --config=$(ARCH)
	@rm -f $@
	cp $(SRCTREE)/bazel-bin/init/init $@
