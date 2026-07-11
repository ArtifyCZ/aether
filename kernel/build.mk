all: $(OUT)/kernel

TARGET_TRIPLE=$(ARCH)-unknown-none

.PHONY: $(OUT)/$(TARGET_TRIPLE)/debug/kernel
$(OUT)/$(TARGET_TRIPLE)/debug/kernel:
	CARGO_TARGET_DIR=$(OUT) cargo build --bin kernel --target $(TARGET_TRIPLE)

$(OUT)/kernel: $(OUT)/$(TARGET_TRIPLE)/debug/kernel
	@rm -f $@
	cp $< $@
