#!/bin/bash
IMG_PATH=$1
ARCH=$2
BIOS=$3
DEBUG=$4
QEMU="qemu-system-$ARCH"
QEMUFLAGS=""
QEMUFLAGS="$QEMUFLAGS -no-reboot"

if [ "$DEBUG" = "DEBUG" ]; then
    QEMUFLAGS="$QEMUFLAGS -s -S"
fi

if [ "$ARCH" = "x86_64" ]; then
    QEMUFLAGS="$QEMUFLAGS -serial stdio -cdrom $IMG_PATH"
elif [ "$ARCH" = "aarch64" ]; then
    QEMUFLAGS="$QEMUFLAGS -M virt,highmem=on,gic-version=2"
    QEMUFLAGS="$QEMUFLAGS -cpu cortex-a57 -m 2G"
    QEMUFLAGS="$QEMUFLAGS -bios $BIOS"
    QEMUFLAGS="$QEMUFLAGS -drive file=$IMG_PATH,if=none,format=raw,id=hd0,readonly=on"
    QEMUFLAGS="$QEMUFLAGS -device virtio-blk-device,drive=hd0"
    QEMUFLAGS="$QEMUFLAGS -device ramfb"
    QEMUFLAGS="$QEMUFLAGS -device qemu-xhci -device usb-kbd"
    QEMUFLAGS="$QEMUFLAGS -chardev stdio,id=con0 -serial chardev:con0"
else
    echo "Unsupported architecture: $ARCH"
    exit 1
fi

exec $QEMU $QEMUFLAGS
