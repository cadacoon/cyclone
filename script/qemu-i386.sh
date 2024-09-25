#!/bin/sh

cargo build -p meerkat_kernel --target i386-unknown-meerkat.json

kernel=target/i386-unknown-meerkat/debug/meerkat_kernel
objcopy -O binary $kernel $kernel.bin
qemu-system-i386 \
    -m 8G \
    -kernel $kernel.bin \
    -no-reboot -no-shutdown -s -d in_asm,int \
#    -accel kvm -smp 4 \
