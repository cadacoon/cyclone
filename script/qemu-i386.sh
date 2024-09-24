#!/bin/sh

kernel=target/i386-unknown-cyclone/debug/cyclone_kernel

cargo build -p cyclone_kernel --target i386-unknown-cyclone.json
objcopy -O binary $kernel $kernel.bin
qemu-system-i386 \
    -m 8G \
    -kernel $kernel.bin \
    -no-reboot -no-shutdown -s -d in_asm,int \
#    -accel kvm -smp 4 \
