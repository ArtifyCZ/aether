imagePkgPackages := $(patsubst %,pkg/%/install,$(PACKAGES))

$(shell mkdir -p $(SYSROOT))

$(OUT)/image/aether.tar: $(imagePkgPackages) $(OUT)/image/initrd.tar | $(OUT)/image/
	@echo "  PACK    $@"
	cp $(OUT)/image/initrd.tar $(SYSROOT)/boot/initrd.tar
	cd $(SYSROOT) && COPYFILE_DISABLE=1 tar --format=ustar -cf $(abspath $@) *
