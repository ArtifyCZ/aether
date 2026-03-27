load("@rules_cc//cc:cc_binary.bzl", "cc_binary")
load("@rules_rust//rust:defs.bzl", "rust_binary")

def _aether_transition_impl(settings, attr):
    platforms = settings.get("//command_line_option:platforms", [])
    if not platforms:
        return {}

    platform_str = str(platforms[0])
    is_x86 = "x86_64" in platform_str
    is_aarch64 = "aarch64" in platform_str

    target_p = ""
    if attr.mode == "kernel":
        target_p = "//platforms:kernel_x86_64" if is_x86 else "//platforms:kernel_aarch64"
    else:
        target_p = "//platforms:user_x86_64" if is_x86 else "//platforms:user_aarch64"

    # Identity check to prevent collisions and save performance
    if target_p in platform_str:
        return {}

    if "macos" in platform_str or "linux" in platform_str:
        return {}

    return {"//command_line_option:platforms": [target_p]}

aether_transition = transition(
    implementation = _aether_transition_impl,
    inputs = ["//command_line_option:platforms"],
    outputs = ["//command_line_option:platforms"],
)

def _aether_binary_wrapper_impl(ctx):
    actual_target = ctx.attr.actual[0]

    # Get the original executable
    orig_exe = actual_target[DefaultInfo].files_to_run.executable

    # Determine the output filename (pretty name)
    out_name = ctx.attr.binary_name if ctx.attr.binary_name else ctx.label.name
    out_exe = ctx.actions.declare_file(out_name)

    # Create the symlink
    ctx.actions.symlink(
        output = out_exe,
        target_file = orig_exe,
        is_executable = True,
    )

    return [
        DefaultInfo(
            executable = out_exe,
            files = depset([out_exe]),
            runfiles = actual_target[DefaultInfo].default_runfiles,
        ),
    ]

_aether_binary_wrapper = rule(
    implementation = _aether_binary_wrapper_impl,
    executable = True,
    attrs = {
        "actual": attr.label(cfg = aether_transition),
        "mode": attr.string(values = ["user", "kernel"], default = "user"),
        "binary_name": attr.string(),
        "_allowlist_function_transition": attr.label(
            default = "@bazel_tools//tools/allowlists/function_transition_allowlist",
        ),
    },
)

# Shared logic for Rust binaries
def _aether_generic_rust_binary(name, mode, **kwargs):
    internal_name = name + "_bin_internal"
    visibility = kwargs.pop("visibility", None)
    testonly = kwargs.pop("testonly", False)
    pretty_binary_name = kwargs.pop("binary_name", None)

    # Collision fix for Rust: use binary_name for the internal file
    internal_binary_filename = (pretty_binary_name or name) + "_bin_actual"

    if "crate_name" not in kwargs:
        kwargs["crate_name"] = name + "_actual"

    rust_binary(
        name = internal_name,
        visibility = ["//visibility:private"],
        testonly = testonly,
        binary_name = internal_binary_filename,
        **kwargs
    )

    _aether_binary_wrapper(
        name = name,
        actual = ":" + internal_name,
        mode = mode,
        binary_name = pretty_binary_name,
        visibility = visibility,
        testonly = testonly,
    )

def _aether_generic_cc_binary(name, mode, **kwargs):
    # The internal target name will be the filename for cc_binary
    internal_name = name + "_bin_actual"

    visibility = kwargs.pop("visibility", None)
    testonly = kwargs.pop("testonly", False)
    pretty_binary_name = kwargs.pop("binary_name", None)

    # Create the real cc_binary
    # cc_binary uses 'name' as the output filename, so it will produce 'hello_world_bin_actual'
    cc_binary(
        name = internal_name,
        visibility = ["//visibility:private"],
        testonly = testonly,
        **kwargs
    )

    # Create the wrapper which symlinks 'hello_world_bin_actual' -> 'hello_world'
    _aether_binary_wrapper(
        name = name,
        actual = ":" + internal_name,
        mode = mode,
        binary_name = pretty_binary_name,
        visibility = visibility,
        testonly = testonly,
    )

# Public API
def user_rust_binary(name, **kwargs):
    _aether_generic_rust_binary(name, mode = "user", **kwargs)

def kernel_rust_binary(name, **kwargs):
    _aether_generic_rust_binary(name, mode = "kernel", **kwargs)

def user_cc_binary(name, **kwargs):
    _aether_generic_cc_binary(name, mode = "user", **kwargs)
