load("//3rdparty/crates:crates.bzl", _third_party_crates = "crate_repositories")

def _third_party_deps_impl(module_ctx):
    direct_deps = []
    direct_deps.extend(_third_party_crates())
    return module_ctx.extension_metadata(
        root_module_direct_deps = [repo.repo for repo in direct_deps],
        root_module_direct_dev_deps = [],
    )

third_party_deps = module_extension(
    implementation = _third_party_deps_impl,
)
