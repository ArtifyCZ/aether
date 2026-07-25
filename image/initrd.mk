initrdStagingDir := $(OUT)/image/initrd-staging

initrdFiles :=
# @TODO: enable once the package's are enabled again
# initrdFiles += bin/hello_world
# initrdFiles += bin/init
# @TODO: remove this once there is another file in initrd
# This file is just a stub so that at least something is in initrd
initrdFiles += boot/limine/limine.conf

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
