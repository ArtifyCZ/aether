all: $(OUT)/hello_world

TARGET_TRIPLE=$(ARCH)-unknown-none

.PHONY: $(OUT)/$(TARGET_TRIPLE)/debug/hello_world
$(OUT)/$(TARGET_TRIPLE)/debug/hello_world:
	CARGO_TARGET_DIR=$(OUT) cargo build --bin hello_world --target $(TARGET_TRIPLE)

$(OUT)/hello_world: $(OUT)/$(TARGET_TRIPLE)/debug/hello_world
	@rm -f $@
	cp $< $@
