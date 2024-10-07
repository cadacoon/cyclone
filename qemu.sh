#!/bin/sh

target=i386

kernel=target/$target-unknown-meerkat/debug/krnl
cargo build -p krnl --target krnl/$target-unknown-meerkat.json
objcopy -O binary $kernel $kernel.bin

qemu-system-$target \
    -m 8G -smp 4 \
    -kernel $kernel.bin \
    -no-reboot -no-shutdown -s -d in_asm,int \
    -drive id=disk,file=target/qemu.img,if=none \
    -device ahci,id=ahci \
    -device ide-hd,drive=disk,bus=ahci.0
