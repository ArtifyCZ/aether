imagePkgStagingDir := $(OUT)/image/pkg/staging

limineCache := $(BUILD)/_cache/limine-v10.x-binary
limineArtifacts := \
    BOOTAA64.EFI BOOTIA32.EFI BOOTRISCV64.EFI BOOTX64.EFI \
    limine-bios-cd.bin limine-bios.sys limine-uefi-cd.bin

$(OUT)/image/pkg/kernel: FORCE
	@echo "  BAZEL    //kernel"
	@cd $(SRCTREE) && bazel build //kernel --config=$(ARCH)
	@mkdir -p $(@D)
	@rm -f $@
	@cp $(SRCTREE)/bazel-bin/kernel/entry/kernel $@

$(OUT)/image/pkg/initrd.tar: FORCE
	@echo "  BAZEL    //image/initrd"
	@cd $(SRCTREE) && bazel build //image/initrd --config=$(ARCH)
	@mkdir -p $(@D)
	@rm -f $@
	@cp $(SRCTREE)/bazel-bin/image/initrd/initrd.tar $@

imagePkgAdditionalFiles := \
    $(SRCTREE)/image/pkg/boot/Mik_8x16.psf \
    $(SRCTREE)/image/pkg/boot/limine/limine.conf \
    $(addprefix $(limineCache)/,$(limineArtifacts)) \
    $(OUT)/image/pkg/kernel \
    $(OUT)/image/pkg/initrd.tar

$(OUT)/image/pkg/aether.tar: $(imagePkgAdditionalFiles)
	@echo "  PACK    $@"
	@mkdir -p $(@D)
	@rm -f $@
	@rm -rf $(imagePkgStagingDir)
	@mkdir -p $(imagePkgStagingDir)/EFI/BOOT $(imagePkgStagingDir)/boot/limine
	@for f in $(limineArtifacts); do \
	    cp $(limineCache)/$$f $(imagePkgStagingDir)/EFI/BOOT/$$f; \
	    cp $(limineCache)/$$f $(imagePkgStagingDir)/boot/limine/$$f; \
	done
	@cp $(SRCTREE)/image/pkg/boot/Mik_8x16.psf $(imagePkgStagingDir)/boot/
	@cp $(SRCTREE)/image/pkg/boot/limine/limine.conf $(imagePkgStagingDir)/boot/limine/
	@cp $(OUT)/image/pkg/kernel $(imagePkgStagingDir)/boot/
	@cp $(OUT)/image/pkg/initrd.tar $(imagePkgStagingDir)/boot/
	@tar -cf $@ -C $(imagePkgStagingDir) .
