kernel_platform_module_dir := $(call get_current_dir)
kernel_platform_module_rel_dir := $(call relative_path_from_srctree,$(kernel_platform_module_dir))
kernel_platform_module_build_dir := $(BUILD)/$(kernel_platform_module_rel_dir)

export KERNEL_PLATFORM_MODULE__LIB_KERNEL_PLATFORM_A := $(kernel_platform_module_build_dir)/libkernel_platform.$(ARCH).a

$(KERNEL_PLATFORM_MODULE__LIB_KERNEL_PLATFORM_A):
	$(MAKE) -C $(kernel_platform_module_dir) $@
