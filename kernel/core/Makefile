include ../../common.mk

RUST_TARGET := $(ARCH)-unknown-none

CARGOFLAGS += --target $(RUST_TARGET)
CARGOFLAGS += --profile dev

kernel_dir := $(realpath $(CURDIR))

.PHONY: $(KERNEL_CORE_MODULE__LIB_KERNEL_CORE_A)
$(KERNEL_CORE_MODULE__LIB_KERNEL_CORE_A):
	cargo build $(CARGOFLAGS)
	mkdir -p $(dir $(KERNEL_CORE_MODULE__LIB_KERNEL_CORE_A))
	cp -v $(kernel_dir)/target/$(RUST_TARGET)/debug/libkernel_core.a $(KERNEL_CORE_MODULE__LIB_KERNEL_CORE_A)

clean::
	cd $(kernel_dir) && cargo clean
