# Check if the architecture is supported.
ifeq ($(filter $(ARCH),aarch64 x86_64),)
    $(error Architecture $(ARCH) not supported)
endif

platform_dir := $(realpath $(call get_current_dir))
platform_rel_dir := $(call relative_path_from_srctree,$(platform_dir))
platform_inc_dir := $(platform_dir)/include
platform_src_dir := $(platform_dir)/src
platform_obj_build_dir := $(BUILD)/$(call relative_path_from_srctree,$(platform_dir))/objs_$(ARCH)

PLATFORM_SRC_FILES :=
PLATFORM_ASM_FILES :=

# Common files
PLATFORM_SRC_FILES += $(wildcard $(platform_src_dir)/*.c)
PLATFORM_SRC_FILES += $(wildcard $(platform_src_dir)/common/**/*.c)

# Arch-specific files
PLATFORM_SRC_FILES += $(wildcard $(platform_src_dir)/arch/$(ARCH)/*.c)
PLATFORM_ASM_FILES += $(wildcard $(platform_src_dir)/arch/$(ARCH)/*.asm) $(wildcard $(platform_src_dir)/arch/$(ARCH)/*.S)

# Serial driver files
PLATFORM_SRC_FILES += $(wildcard $(platform_src_dir)/drivers/serial/$(ARCH)/*.c)
PLATFORM_SRC_FILES += $(wildcard $(platform_src_dir)/drivers/serial/$(ARCH)/**.c)

PLATFORM_ASM_FILES += $(wildcard $(platform_src_dir)/*.$(ARCH).asm) $(wildcard $(platform_src_dir)/**/*.$(ARCH).asm)
PLATFORM_ASM_FILES += $(wildcard $(platform_src_dir)/*.$(ARCH).S) $(wildcard $(platform_src_dir)/**/*.$(ARCH).S)

PLATFORM_OBJS=
PLATFORM_OBJS += $(patsubst $(platform_src_dir)/%.c,$(platform_obj_build_dir)/%.$(ARCH).o,$(PLATFORM_SRC_FILES))
PLATFORM_OBJS += $(patsubst $(platform_src_dir)/%.$(ARCH).asm,$(platform_obj_build_dir)/%.$(ARCH).o,$(PLATFORM_ASM_FILES))
PLATFORM_OBJS += $(patsubst $(platform_src_dir)/%.$(ARCH).S,$(platform_obj_build_dir)/%.$(ARCH).o,$(PLATFORM_ASM_FILES))
PLATFORM_OBJS += $(patsubst $(platform_src_dir)/arch/$(ARCH)/%.S,$(platform_obj_build_dir)/arch/$(ARCH)/%.$(ARCH).o,$(PLATFORM_ASM_FILES))
PLATFORM_OBJS += $(patsubst $(platform_src_dir)/arch/$(ARCH)/%.asm,$(platform_obj_build_dir)/arch/$(ARCH)/%.$(ARCH).o,$(PLATFORM_ASM_FILES))


INCLUDE_HEADERS := $(platform_inc_dir)

INCLUDE_HEADERS += $(DEPS)/limine-protocol/include
INCLUDE_HEADERS += $(DEPS)/freestnd-c-hdrs/include


CFLAGS :=
CFLAGS += $(addprefix -I ,$(INCLUDE_HEADERS))
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


$(platform_obj_build_dir)/%.x86_64.o: $(platform_dir)/src/%.x86_64.asm $(BUILD)
	(mkdir -p $(dir $@) || true)
	$(NASM) $(NASMFLAGS) $< -o $@

$(platform_obj_build_dir)/%.aarch64.o: $(platform_dir)/src/%.aarch64.S $(BUILD)
	(mkdir -p $(dir $@) || true)
	$(AARCH64_ELF_AS) $< -o $@

$(platform_obj_build_dir)/arch/x86_64/%.x86_64.o: $(platform_dir)/src/arch/x86_64/%.asm $(BUILD)
	(mkdir -p $(dir $@) || true)
	$(NASM) $(NASMFLAGS) $< -o $@

$(platform_obj_build_dir)/arch/aarch64/%.aarch64.o: $(platform_dir)/src/arch/aarch64/%.S $(BUILD)
	(mkdir -p $(dir $@) || true)
	$(AARCH64_ELF_AS) $< -o $@

$(platform_obj_build_dir)/%.$(ARCH).o: $(platform_dir)/src/%.c $(BUILD)
	(mkdir -p $(dir $@) || true)
	$(CC) $(CFLAGS) -c $< -o $@


$(BUILD)/libplatform.$(ARCH).a: $(BUILD) $(DEPS) $(PLATFORM_OBJS)
	(rm $(BUILD)/libplatform.$(ARCH).a || true)
	ar rcs $(BUILD)/libplatform.$(ARCH).a $(PLATFORM_OBJS)


$(DEPS): $(DEPS)/limine-protocol $(DEPS)/freestnd-c-hdrs


$(DEPS)/limine-protocol: repo = https://codeberg.org/Limine/limine-protocol.git
$(DEPS)/limine-protocol: commit = 42e836e30242c2c14f889fd76c6f9a57b0c18ec2

$(DEPS)/freestnd-c-hdrs: repo = https://codeberg.org/OSDev/freestnd-c-hdrs-0bsd.git
$(DEPS)/freestnd-c-hdrs: commit = 097259a899d30f0a4b7a694de2de5fdda942e923


$(DEPS)/%:
	rm -rf $@
	git clone $(repo) $@
	git -C $@ -c advice.detachedHead=false checkout $(commit)
