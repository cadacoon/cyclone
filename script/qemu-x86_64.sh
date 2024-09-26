#!/bin/sh

cargo build -p krnl --target x86_64-unknown-meerkat.json

kernel=target/x86_64-unknown-meerkat/debug/krnl
objcopy -O binary $kernel $kernel.bin
qemu-system-x86_64 \
    -m 8G \
    -kernel $kernel.bin \
    -no-reboot -no-shutdown -s -d in_asm,int \
#    -accel kvm -smp 4 \
