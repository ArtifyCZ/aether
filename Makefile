this_makefile := $(abspath $(lastword $(MAKEFILE_LIST)))

export O ?= $(CURDIR)
export ARCH ?= x86_64
export DIST ?= $(O)/dist

export CURDIR := $(abspath .)
export SRCTREE := $(abspath $(dir $(this_makefile)))
export BUILD := $(O)/build
export OUT := $(BUILD)/$(ARCH)

PHONY :=
__default: all

ifeq ($(ARCH),host)
diskImageExt :=
else ifeq ($(ARCH),x86_64)
diskImageExt := iso
else ifeq ($(ARCH),aarch64)
diskImageExt := img
else
$(error Disk image file extension not configured for ARCH: $(ARCH))
endif

diskImageBasename := aether-$(ARCH).$(diskImageExt)

PHONY += FORCE
FORCE: ;

PHONY += all
all: $(DIST)/$(diskImageBasename)

ifneq ($(ARCH),host)
$(BUILD)/host/%: FORCE
	@echo "  RECURSE MAKE HOST $@"
	@$(MAKE) $@ ARCH=host
endif

include $(SRCTREE)/3rdparty/package.mk
include $(SRCTREE)/image/image.mk
include $(SRCTREE)/mk/compile_commands.json.mk
include $(SRCTREE)/mk/qemu.mk
include $(SRCTREE)/mk/rust-project.json.mk

%/: # A pattern rule to create directories
	@mkdir -p $@

clean:
	@echo "  CLEAN    $(BUILD)"
	@rm -rf $(BUILD)
	@echo "  CLEAN    $(DIST)"
	@rm -rf $(DIST)
	@echo "  CLEAN    compile_commands.json"
	@rm -rf compile_commands.json
	@echo "  CLEAN    rust-project.json"
	@rm -rf rust-project.json

.PHONY: $(PHONY)
