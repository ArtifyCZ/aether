# Check if the architecture is supported.
ifeq ($(filter $(ARCH),aarch64 x86_64),)
    $(error Architecture $(ARCH) not supported)
endif

init_dir := $(realpath $(call get_current_dir))
init_rel_dir := $(call relative_path_from_srctree,$(init_dir))
init_src_dir := $(init_dir)/src
init_obj_build_dir := $(BUILD)/$(call relative_path_from_srctree,$(init_dir))/objs_$(ARCH)

INIT_SRC_FILES :=
INIT_SRC_FILES += $(wildcard $(init_src_dir)/*.c)

# Driver files
INIT_SRC_FILES += $(wildcard $(init_src_dir)/drivers/*/$(ARCH)/*.c)

INIT_ASM_FILES :=
INIT_ASM_FILES += $(wildcard $(init_src_dir)/*.$(ARCH).asm)
INIT_ASM_FILES += $(wildcard $(init_src_dir)/*.$(ARCH).S)

INIT_OBJS :=
INIT_OBJS += $(patsubst $(init_src_dir)/%.c,$(init_obj_build_dir)/%.$(ARCH).o,$(INIT_SRC_FILES))
INIT_OBJS += $(patsubst $(init_src_dir)/%.$(Arch).asm,$(init_obj_build_dir)/%.$(ARCH).o,$(INIT_ASM_FILES))
INIT_OBJS += $(patsubst $(init_src_dir)/%.$(Arch).S,$(init_obj_build_dir)/%.$(ARCH).o,$(INIT_ASM_FILES))


init_inc_headers := $(DEPS)/freestnd-c-hdrs/include
init_inc_headers += $(init_dir)/include


CFLAGS :=
CFLAGS += -g -O0 -pipe
CFLAGS += -target $(ARCH)-none-elf
CFLAGS += -Wall \
	-Wextra \
	-std=gnu11 \
	-nostdinc \
	-ffreestanding \
	-fno-stack-protector \
	-fno-stack-check \
	-mgeneral-regs-only \
	-mno-red-zone \
	-fno-lto \
	-fno-PIC \
	-ffunction-sections \
	-fdata-sections


ifeq ($(ARCH),x86_64)
	NASMFLAGS += -f elf64
else ifeq ($(ARCH),aarch64)
else
$(error Compilation of assembly not configured for architecture $(ARCH) yet)
endif


$(init_obj_build_dir)/%.x86_64.o: $(init_src_dir)/%.x86_64.asm $(BUILD)
	(mkdir -p $(dir $@) || true)
	$(NASM) $(NASMFLAGS) $< -o $@

$(init_obj_build_dir)/%.aarch64.o: $(init_src_dir)/%.aarch64.S $(BUILD)
	(mkdir -p $(dir $@) || true)
	$(AARCH64_ELF_AS) $< -o $@


$(init_obj_build_dir)/%.$(ARCH).o: $(init_src_dir)/%.c $(BUILD)
	(mkdir -p $(dir $@) || true)
	$(CC) $(CFLAGS) $(addprefix -I ,$(init_inc_headers)) -c $< -o $@


$(BUILD)/init.$(ARCH).elf: $(BUILD) $(INIT_OBJS)
	$(LD) -Ttext 0x400000 $(INIT_OBJS) -o $(BUILD)/init.$(ARCH).elf
