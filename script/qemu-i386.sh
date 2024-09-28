#!/bin/sh

cargo build -p krnl --target i386-unknown-meerkat.json

kernel=target/i386-unknown-meerkat/debug/krnl
objcopy -O binary $kernel $kernel.bin
qemu-system-i386 \
    -m 8G \
    -kernel $kernel.bin \
    -no-reboot -no-shutdown -s -d in_asm,int \
    -drive file=target/qemu.img,format=raw \
    -device virtio-net,netdev=vmnic -netdev user,id=vmnic \
    -machine pcspk-audiodev=speaker -audiodev pa,id=speaker \
    -accel kvm -smp 4
