initrdStagingDir := $(OUT)/image/initrd/staging

$(OUT)/image/initrd/hello_world: FORCE
	@echo "  BAZEL    //hello_world"
	@cd $(SRCTREE) && bazel build //hello_world --config=$(ARCH)
	@mkdir -p $(@D)
	@rm -f $@
	@cp $(SRCTREE)/bazel-bin/hello_world/hello_world $@

$(OUT)/image/initrd/init: FORCE
	@echo "  BAZEL    //init"
	@cd $(SRCTREE) && bazel build //init --config=$(ARCH)
	@mkdir -p $(@D)
	@rm -f $@
	@cp $(SRCTREE)/bazel-bin/init/init $@

initrdFiles := \
    $(OUT)/image/initrd/hello_world \
    $(OUT)/image/initrd/init

$(OUT)/image/initrd/initrd.tar: $(initrdFiles)
	@echo "  PACK    $@"
	@mkdir -p $(@D)
	@rm -f $@
	@rm -rf $(initrdStagingDir)
	@mkdir -p $(initrdStagingDir)/bin
	@cp $(OUT)/image/initrd/hello_world $(initrdStagingDir)/bin/
	@cp $(OUT)/image/initrd/init $(initrdStagingDir)/bin/
	@COPYFILE_DISABLE=1 tar --format=ustar -cf $@ -C $(initrdStagingDir) bin
