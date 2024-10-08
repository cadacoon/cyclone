#!/bin/sh

target=i386

kernel=target/$target-unknown-none/debug/krnl
cargo build -p krnl --target krnl/$target-unknown-none.json || exit 1
objcopy -O binary $kernel $kernel.bin || exit 1

qemu_args=(-kernel $kernel.bin)

qemu_args+=(-m 8G -smp 4)
qemu_args+=(-accel kvm)

qemu_args+=(-no-reboot -no-shutdown -s)
qemu_args+=(-display none -serial stdio)

qemu_args+=(-drive id=disk,file=target/qemu.img,if=none)
qemu_args+=(-device ahci,id=ahci)
qemu_args+=(-device ide-hd,drive=disk,bus=ahci.0)

qemu-system-$target "${qemu_args[@]}"
