all: $(OUT)/init

TARGET_TRIPLE=$(ARCH)-unknown-none

.PHONY: $(OUT)/$(TARGET_TRIPLE)/debug/init
$(OUT)/$(TARGET_TRIPLE)/debug/init:
	CARGO_TARGET_DIR=$(OUT) cargo build --bin init --target $(TARGET_TRIPLE)

$(OUT)/init: $(OUT)/$(TARGET_TRIPLE)/debug/init
	@rm -f $@
	cp $< $@
