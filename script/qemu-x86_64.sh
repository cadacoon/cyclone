#!/bin/sh

kernel = target/x86_64-unknown-cyclone/debug/cyclone_kernel

cargo build -p cyclone_kernel --target x86_64-unknown-cyclone.json
objcopy -O binary $kernel $kernel.bin
qemu-system-x86_64 \
    -m 8G \
    -kernel $kernel.bin \
    -no-reboot -no-shutdown -s -d in_asm,int \
#    -accel kvm -smp 4 \
