load("@bazel_tools//tools/build_defs/cc:action_names.bzl", "ACTION_NAMES")
load("@bazel_tools//tools/cpp:cc_toolchain_config_lib.bzl", "feature", "flag_group", "flag_set", "tool_path", "action_config", "tool")
load("@rules_cc//cc:defs.bzl", "cc_common")

all_compile_actions = [
    ACTION_NAMES.c_compile,
    ACTION_NAMES.cpp_compile,
    ACTION_NAMES.assemble,
    ACTION_NAMES.preprocess_assemble,
]

all_link_actions = [
    ACTION_NAMES.cpp_link_executable,
    ACTION_NAMES.cpp_link_dynamic_library,
    ACTION_NAMES.cpp_link_nodeps_dynamic_library,
]

def _impl(ctx):
    # Get the absolute path to ld.lld from the toolchain
    # This will be used in the linker wrapper
    ld_path = ctx.file.ld.path

    builtin_include_dirs = []
    if ctx.attr.cxx_builtin_include_directories:
        for f in ctx.attr.cxx_builtin_include_directories[DefaultInfo].files.to_list():
            # We want the directory, not the file.
            # For "lib/clang/19/include/stdarg.h", we want "external/toolchains_llvm++.../lib/clang/19/include"
            # However, the compiler usually just needs the logical path from the execroot.
            d = f.short_path

            # Logic to find the 'include' root
            if d.find("include") != -1:
                include_root = d.split("include")[0] + "include"
                if include_root not in builtin_include_dirs:
                    builtin_include_dirs.append(include_root)

    tool_paths = [
        tool_path(name = "gcc", path = ctx.file.gcc.path),
        # Point directly to ld.lld for linking
        tool_path(name = "ld", path = ld_path),
        tool_path(name = "ar", path = ctx.file.ar.path),
        tool_path(name = "nm", path = ctx.file.nm.path),
        tool_path(name = "objcopy", path = ctx.file.objcopy.path),
        tool_path(name = "objdump", path = ctx.file.objdump.path),
        tool_path(name = "strip", path = ctx.file.strip.path),
        tool_path(name = "cpp", path = ctx.file.gcc.path),
    ]

    # Mandatory flags for a freestanding kernel
    common_kernel_flags = [
        "-Wall",
        "-Wextra",
        "-nostdinc",
        "-ffreestanding",
        "-fno-stack-protector",
        "-fno-stack-check",
        "-mgeneral-regs-only",
        "-mno-red-zone",
        "-fno-lto",
        "-fno-PIC",
        "-ffunction-sections",
        "-fdata-sections",
    ]

    c_only_flags = ["-std=gnu11"]
    cpp_only_flags = ["-std=c++20", "-fno-exceptions", "-fno-rtti"]

    # Architecture specific (e.g., x86_64)
    linker_target_flags = []
    if ctx.attr.cpu == "x86_64":
        common_kernel_flags += ["-target", "x86_64-none-elf"]
        common_kernel_flags += ["-D__x86_64__"]
    elif ctx.attr.cpu == "aarch64":
        common_kernel_flags += ["-target", "aarch64-none-elf"]
        common_kernel_flags += ["-D__aarch64__"]

    features = [
        feature(
            name = "kernel_compile_flags",
            enabled = True,
            flag_sets = [
                flag_set(
                    actions = all_compile_actions,
                    flag_groups = [flag_group(flags = common_kernel_flags)],
                ),
                flag_set(
                    actions = [ACTION_NAMES.c_compile],
                    flag_groups = [flag_group(flags = c_only_flags)],
                ),
                flag_set(
                    actions = [ACTION_NAMES.cpp_compile],
                    flag_groups = [flag_group(flags = cpp_only_flags)],
                ),
            ],
        ),
        feature(name = "targets_windows", enabled = False),
        feature(name = "copy_dynamic_libraries_to_binary", enabled = False),
        feature(name = "supports_pic", enabled = False),
        feature(name = "strip_debug_symbols", enabled = False),
    ]

    action_configs = [
        action_config(
            action_name = ACTION_NAMES.cpp_link_executable,
            enabled = True,
            tools = [tool(path = ld_path)],
        ),
        action_config(
            action_name = ACTION_NAMES.cpp_link_dynamic_library,
            enabled = True,
            tools = [tool(path = ld_path)],
        ),
        action_config(
            action_name = ACTION_NAMES.cpp_link_nodeps_dynamic_library,
            enabled = True,
            tools = [tool(path = ld_path)],
        ),
    ]

    return cc_common.create_cc_toolchain_config_info(
        ctx = ctx,
        action_configs = action_configs,
        features = features,
        toolchain_identifier = "kernel-" + ctx.attr.cpu,
        host_system_name = "local",
        target_system_name = "freestanding",
        target_cpu = ctx.attr.cpu,
        target_libc = "none",
        compiler = "clang",
        abi_version = "unknown",
        abi_libc_version = "unknown",
        tool_paths = tool_paths,
        cxx_builtin_include_directories = builtin_include_dirs,
    )

cc_toolchain_config = rule(
    implementation = _impl,
    attrs = {
        "cpu": attr.string(mandatory = True),
        "cxx_builtin_include_directories": attr.label(),
        "gcc": attr.label(allow_single_file = True),
        "ld": attr.label(allow_single_file = True),
        "ar": attr.label(allow_single_file = True),
        "nm": attr.label(allow_single_file = True),
        "objcopy": attr.label(allow_single_file = True),
        "objdump": attr.label(allow_single_file = True),
        "strip": attr.label(allow_single_file = True),
    },
)
