$(BUILD)/_cache/limine-v10.x-binary:
	@echo "  GIT CLONE  $@"
	@mkdir -p $@
	@git clone https://github.com/limine-bootloader/limine.git $@ --branch v10.x-binary --depth 1


ifeq ($(ARCH),host)

$(OUT)/bin/limine: $(BUILD)/_cache/limine-v10.x-binary | $(OUT)/bin/
	@echo "  RECURSE  $<"
	@$(MAKE) -C $< limine
	@ln -sf $</limine $@

endif
