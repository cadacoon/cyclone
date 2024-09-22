#!/bin/sh

objcopy -O binary $1 $1.bin

qemu-system-x86_64 \
    -m 8G \
    -kernel $1.bin \
    -no-reboot -no-shutdown -s -d in_asm,int \
#   -accel kvm -smp cores=4 -m 4G \
