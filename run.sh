#!/bin/sh

objcopy -O binary $1 $1.bin

qemu-system-x86_64 \
    -accel kvm -smp cores=4 -m 4G \
    -kernel $1.bin
