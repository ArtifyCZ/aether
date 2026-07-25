all: $(OUT)/atom

TARGET_TRIPLE=$(ARCH)-unknown-none

.PHONY: $(OUT)/$(TARGET_TRIPLE)/debug/atom
$(OUT)/$(TARGET_TRIPLE)/debug/atom:
	CARGO_TARGET_DIR=$(OUT) cargo build --bin atom --target $(TARGET_TRIPLE)

$(OUT)/atom: $(OUT)/$(TARGET_TRIPLE)/debug/atom
	@rm -f $@
	cp $< $@
