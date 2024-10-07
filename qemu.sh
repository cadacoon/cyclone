#!/bin/sh

cargo build -p krnl --target krnl/x86_64-unknown-meerkat.json

objcopy -O binary target/x86_64-unknown-meerkat/debug/krnl  target/x86_64-unknown-meerkat/debug/krnl.bin
qemu-system-x86_64 \
    -m 8G -smp 4 \
    -kernel target/x86_64-unknown-meerkat/debug/krnl.bin \
    -no-reboot -no-shutdown -s -d in_asm,int \
    -drive id=disk,file=target/qemu.img,if=none \
    -device ahci,id=ahci \
    -device ide-hd,drive=disk,bus=ahci.0
