initrdStagingDir := $(OUT)/image/initrd-staging

initrdFiles :=
initrdFiles += bin/hello_world
initrdFiles += bin/init

initrdSysrootFiles := $(initrdFiles:%=$(SYSROOT)/%)

$(OUT)/image/initrd.tar: $(initrdSysrootFiles)
	@echo "  PACK    $@"
	@mkdir -p $(@D)
	rm -f $@
	rm -rf $(initrdStagingDir)
	for file in $(initrdFiles); do \
		mkdir -p $(initrdStagingDir)/$$(dirname $$file); \
		cp $(SYSROOT)/$$file $(initrdStagingDir)/$$file; \
	done
	cd $(initrdStagingDir) && COPYFILE_DISABLE=1 tar --format=ustar -cf $(abspath $@) *
