#!/bin/bash

ARCH=$1

if [ "$ARCH" = "x86_64" ]; then
    echo "Synchronizing for x86_64"
elif [ "$ARCH" = "aarch64" ]; then
    echo "Synchronizing for aarch64"
else
    echo "Unsupported architecture: $ARCH"
    exit 1
fi

cd "$BUILD_WORKSPACE_DIRECTORY" || exit 1


python3 tooling/sync_ide/discover_bazel_c_compile_commands.py --config="$ARCH"
bash tooling/sync_ide/discover_bazel_rust_project.sh --bazel_arg=--platforms=//platforms:kernel_"$ARCH"
