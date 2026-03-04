kernel_module_dir := $(call get_current_dir)
kernel_module_rel_dir := $(call relative_path_from_srctree,$(kernel_module_dir))
kernel_module_build_dir := $(BUILD)/$(kernel_module_rel_dir)

export KERNEL_MODULE__KERNEL_ELF := $(kernel_module_build_dir)/kernel.$(ARCH).elf

include $(kernel_module_dir)/core/module.mk
include $(kernel_module_dir)/platform/module.mk

$(KERNEL_MODULE__KERNEL_ELF):
	$(MAKE) -C $(kernel_module_dir) $@
