$(OUT)/aether-x86_64.iso: $(BUILD)/host/bin/limine $(OUT)/image/aether.tar | $(OUT)/ $(OUT)/image/root/
	@echo "  BUILD    $@"
	@tar -xf $(OUT)/image/aether.tar -C $(OUT)/image/root/
	@xorriso -as mkisofs -R -r -J -b boot/limine/limine-bios-cd.bin \
    	-no-emul-boot -boot-load-size 4 -boot-info-table -hfsplus \
    	-apm-block-size 2048 --efi-boot boot/limine/limine-uefi-cd.bin \
    	-efi-boot-part --efi-boot-image --protective-msdos-label \
    	$(OUT)/image/root/ -o $@
