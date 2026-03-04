kernel_core_module_dir := $(call get_current_dir)
kernel_core_module_rel_dir := $(call relative_path_from_srctree,$(kernel_core_module_dir))
kernel_core_module_build_dir := $(BUILD)/$(kernel_core_module_rel_dir)

export KERNEL_CORE_MODULE__LIB_KERNEL_CORE_A := $(kernel_core_module_build_dir)/libkernel_core.$(ARCH).a

$(KERNEL_CORE_MODULE__LIB_KERNEL_CORE_A):
	$(MAKE) -C $(kernel_core_module_dir) $@
