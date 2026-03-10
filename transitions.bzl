
def _platform_transition_impl(settings, attr):
    # This transitions the build to the specific platform string provided in the attribute
    return {"//command_line_option:platforms": [attr.platform]}

_platform_transition = transition(
    implementation = _platform_transition_impl,
    inputs = [],
    outputs = ["//command_line_option:platforms"],
)

def _transition_rule_impl(ctx):
    # Just return the files from the underlying target
    target = ctx.attr.target[0]
    return [DefaultInfo(files = target[DefaultInfo].files)]

# A wrapper rule that forces a target to be built for a specific platform
platform_binary_wrapper = rule(
    implementation = _transition_rule_impl,
    attrs = {
        "target": attr.label(cfg = _platform_transition),
        "platform": attr.string(),
        "_allowlist_function_transition": attr.label(
            default = "@bazel_tools//tools/allowlists/function_transition_allowlist"
        ),
    },
)
