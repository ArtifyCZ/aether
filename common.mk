get_current_dir = $(realpath $(dir $(lastword $(MAKEFILE_LIST))))
srctree := $(call get_current_dir)

ifeq ($(MAKELEVEL),0)
ifneq ($(CURDIR),$(srctree))
%:
	@$(MAKE) -C $(srctree) $@
endif
endif

-include local.mk

# --- Toolchain Configuration ---
ARCH ?= x86_64
CC := clang
LD := ld.lld
NASM := nasm
AARCH64_ELF_AS ?= aarch64-linux-gnu-as

# Check if the architecture is supported.
ifeq ($(filter $(ARCH),aarch64 x86_64),)
    $(error Architecture $(ARCH) not supported)
endif

export ARCH
export CC
export LD
export NASM
export AARCH64_ELF_AS

# Fallback QEMU EFI firmware for aarch64
QEMU_AARCH64_BIOS ?= /usr/share/edk2/aarch64/QEMU_EFI.fd
export QEMU_AARCH64_BIOS

export srctree

BUILD := $(abspath $(srctree)/build/$(ARCH))

export BUILD
