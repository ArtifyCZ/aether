$(OUT)/aether-aarch64.img: $(BUILD)/host/bin/limine $(OUT)/image/pkg/aether.tar | $(OUT)/
	@echo "  BUILD    $@"
	@mkdir -p $(OUT)/image/root
	dd if=/dev/zero of=$@ bs=1M count=64
	mformat -i $@ ::
	@tar -xf $(OUT)/image/pkg/aether.tar -C $(OUT)/image/root/
	mcopy -i $@ -s $(OUT)/image/root/* ::
