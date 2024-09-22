// Copyright 2024 Kevin Ludwig
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use core::{arch, cell, mem, ptr, slice};

use alloc::vec;

use crate::{main, mm, multiboot, util};

pub static PHYSMEM: cell::SyncUnsafeCell<[usize; 2048 / usize::BITS as usize]> =
    cell::SyncUnsafeCell::new([0; 2048 / usize::BITS as usize]);
pub static PTL0: cell::SyncUnsafeCell<mm::PageTable> =
    cell::SyncUnsafeCell::new(mm::PageTable::new());
pub static PTL1: cell::SyncUnsafeCell<mm::PageTable> =
    cell::SyncUnsafeCell::new(mm::PageTable::new());

arch::global_asm!(include_str!("i386_entry.S"), options(att_syntax));

#[no_mangle]
#[link_section = ".multiboot.init"]
unsafe fn main_bootstrap(
    _multiboot_magic: u32,
    multiboot_info: &mut multiboot::multiboot_info,
) -> ! {
    // 1. Bootstrap virtual memory
    let ptl1_virt_addr = PTL1.get();
    let ptl1_phys_addr = ptl1_virt_addr.byte_sub(0xF000_0000);
    let ptl1 = &mut *ptl1_phys_addr;
    for frame in 0..1024 {
        ptl1[frame].map(frame); // identity
    }
    let ptl0_virt_addr = PTL0.get();
    let ptl0_phys_addr = ptl0_virt_addr.byte_sub(0xF000_0000);
    let ptl0 = &mut *ptl0_phys_addr;
    ptl0[0x000].map((ptl1_phys_addr as usize) >> 12); // identity
    ptl0[0x3C0].map((ptl1_phys_addr as usize) >> 12); // system & kernel
    ptl0[0x3FF].map((ptl0_phys_addr as usize) >> 12); // self-reference

    // 2. Enable paging
    arch::asm!(
        "mov cr3, {}",
        "mov {tmp}, cr0",
        "or {tmp}, 0x80010000",
        "mov cr0, {tmp}",
        in(reg) ptl0_phys_addr,
        tmp = out(reg) _,
    );

    // 3. Fix addresses
    arch::asm!(
        "mov {tmp}, esp",
        "add {tmp}, 0xF0000000",
        "mov esp, {tmp}",
        "mov {tmp}, ebp",
        "add {tmp}, 0xF0000000",
        "mov ebp, {tmp}",
        tmp = out(reg) _,
    );

    // 4.1 Init physical memory (minimally)
    {
        let mut phys_mem = mm::PHYS_MEM.lock();
        *phys_mem = mm::PhysicalMemory::new(
            unsafe {
                mem::transmute(ptr::slice_from_raw_parts(
                    PHYSMEM.get(),
                    2048 / usize::BITS as usize,
                ))
            },
            2048,
        );
        phys_mem.mark_used(0, 1024); // system & kernel
    }

    // 4.1 Init physical memory by using the E820 memory map (needs to allocate a
    // properly sized bitmap)
    let phys_mem_map = slice::from_raw_parts(
        multiboot_info.mmap_addr as usize as *const multiboot::multiboot_mmap_entry,
        multiboot_info.mmap_length as usize / size_of::<multiboot::multiboot_mmap_entry>(),
    );
    let phys_mem_max: usize = phys_mem_map
        .iter()
        .filter(|phys_mem_entry| phys_mem_entry.type_ == multiboot::MULTIBOOT_MEMORY_AVAILABLE)
        .map(|phys_mem_entry| {
            ((phys_mem_entry.addr + phys_mem_entry.len) / mm::GRANULARITY as u64) as usize
        })
        .max()
        .unwrap();
    let phys_mem_new = mm::PhysicalMemory::new(
        util::bitmap::Bitmap::new(
            vec![usize::MAX; phys_mem_max / usize::BITS as usize].into_boxed_slice(),
        ),
        0,
    );
    {
        let mut phys_mem = mm::PHYS_MEM.lock();
        *phys_mem = phys_mem_new;
        for phys_mem_entry in phys_mem_map {
            if phys_mem_entry.type_ != multiboot::MULTIBOOT_MEMORY_AVAILABLE {
                continue;
            }

            let frame_start = phys_mem_entry.addr / mm::GRANULARITY as u64;
            let frame_end = frame_start + (phys_mem_entry.len / mm::GRANULARITY as u64);
            let frames = frame_end - frame_start;
            if frames == 0 {
                continue;
            }

            phys_mem.mark_free(frame_start as usize, frames as usize);
        }
        phys_mem.mark_used(0, 1024); // system & kernel
    }

    // 5. Call into main
    main();
}
