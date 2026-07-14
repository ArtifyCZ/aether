QEMU ?= qemu-system-$(ARCH)

ifeq ($(ARCH),aarch64)
qemuBios := $(OUT)/3rdparty/edk2/edk2-aarch64-code.fd
else
qemuBios := -
endif

PHONY += qemu
qemu: $(OUT)/$(diskImageBasename) $(if $(filter aarch64,$(ARCH)),$(qemuBios))
	/usr/bin/env bash $(SRCTREE)/run_qemu.sh $< $(ARCH) $(qemuBios)
