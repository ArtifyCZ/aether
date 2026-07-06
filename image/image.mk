include $(SRCTREE)/image/img.mk
include $(SRCTREE)/image/iso.mk
include $(SRCTREE)/image/initrd.mk
include $(SRCTREE)/image/pkg.mk

$(DIST)/aether-$(ARCH).$(diskImageExt): $(OUT)/aether-$(ARCH).$(diskImageExt) | $(DIST)/
	@echo "  COPY      $@"
	@cp $< $@
