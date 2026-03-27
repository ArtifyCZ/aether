load("@rules_rust//rust:defs.bzl", "rust_binary")

def _aether_transition_impl(settings, attr):
    platforms = settings.get("//command_line_option:platforms", [])
    if not platforms:
        return {}

    platform_str = str(platforms[0])

    # SAFETY CHECK:
    # If the current platform is already one of our "User" platforms,
    # return {} to indicate NO transition is needed.
    # This is the most important part to stop the "Conflicting Actions" error.
    if "kernel_x86_64" in platform_str or "kernel_aarch64" in platform_str:
        return {}

    # Also skip if we are on a host OS (for local tests)
    if "macos" in platform_str or "linux" in platform_str:
        return {}

    # Perform the transition based on the CPU
    if "x86_64" in platform_str:
        return {"//command_line_option:platforms": ["//platforms:kernel_x86_64"]}
    elif "aarch64" in platform_str:
        return {"//command_line_option:platforms": ["//platforms:kernel_aarch64"]}

    return {}

aether_transition = transition(
    implementation = _aether_transition_impl,
    inputs = ["//command_line_option:platforms"],
    outputs = ["//command_line_option:platforms"],
)

def _kernel_binary_wrapper_impl(ctx):
    actual_target = ctx.attr.actual[0]

    # Get the original executable from the transitioned target
    # Note: Using .files.to_list()[0] is safer here than .files_to_run.executable
    # in some cross-config scenarios.
    orig_exe = actual_target[DefaultInfo].files.to_list()[0]

    # Declare our public executable
    # If binary_name is provided, use it; otherwise, use the rule name
    out_exe = ctx.actions.declare_file(ctx.label.name)

    # Create the symlink
    ctx.actions.symlink(
        output = out_exe,
        target_file = orig_exe,
        is_executable = True,
    )

    # Return the providers, ensuring we ONLY expose our new file
    return [
        DefaultInfo(
            executable = out_exe,
            files = depset([out_exe]),
            runfiles = actual_target[DefaultInfo].default_runfiles,
        ),
    ]

_kernel_binary_wrapper = rule(
    implementation = _kernel_binary_wrapper_impl,
    executable = True,
    attrs = {
        "actual": attr.label(cfg = aether_transition),
        "_allowlist_function_transition": attr.label(
            default = "@bazel_tools//tools/allowlists/function_transition_allowlist",
        ),
    },
)

def kernel_rust_binary(name, **kwargs):
    # Use a clear, simple suffix that won't confuse Bazel's path normalization
    internal_name = name + "_bin_internal"

    visibility = kwargs.pop("visibility", None)
    testonly = kwargs.pop("testonly", False)

    # Force the crate name to be different so the .rlib doesn't collide
    if "crate_name" not in kwargs:
        kwargs["crate_name"] = name + "_actual"

    # Create the real rust_binary
    rust_binary(
        name = internal_name,
        visibility = ["//visibility:private"],
        testonly = testonly,
        **kwargs
    )

    # Create the wrapper with the actual name
    _kernel_binary_wrapper(
        name = name,
        actual = ":" + internal_name,
        visibility = visibility,
        testonly = testonly,
    )
