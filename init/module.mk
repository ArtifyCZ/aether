init_module_dir := $(call get_current_dir)
init_module_rel_dir := $(call relative_path_from_srctree,$(init_module_dir))
init_module_build_dir := $(BUILD)/$(init_module_rel_dir)

export INIT_MODULE__INIT_ELF := $(init_module_build_dir)/init.$(ARCH).elf

$(INIT_MODULE__INIT_ELF):
	$(MAKE) -C $(init_module_dir) $@
