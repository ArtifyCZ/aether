load("@bazel_tools//tools/build_defs/repo:git.bzl", "git_repository")
load("@bazel_tools//tools/build_defs/repo:utils.bzl", "maybe")
load("//3rdparty/crates:crates.bzl", _third_party_crates = "crate_repositories")

def _third_party_deps_impl(module_ctx):
    direct_deps = []

    maybe(
        git_repository,
        name = "freestnd_c_hdrs",
        remote = "https://github.com/osdev0/freestnd-c-hdrs-0bsd.git",
        commit = "5e4e9e70278fe89ea328d359a58aff4f4a94b165",
        build_file = "//3rdparty:BUILD.freestnd_c_hdrs.bazel",
    )
    direct_deps.append(struct(repo = "freestnd_c_hdrs", is_dev_dep = False))

    maybe(
        git_repository,
        name = "limine_bootloader",
        remote = "https://github.com/limine-bootloader/limine.git",
        branch = "v10.x-binary",
        build_file = "//3rdparty:BUILD.limine_bootloader.bazel",
    )
    direct_deps.append(struct(repo = "limine_bootloader", is_dev_dep = False))

    direct_deps.extend(_third_party_crates())

    return module_ctx.extension_metadata(
        root_module_direct_deps = [repo.repo for repo in direct_deps],
        root_module_direct_dev_deps = [],
    )

third_party_deps = module_extension(
    implementation = _third_party_deps_impl,
)
