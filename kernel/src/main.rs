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

#![no_std]
#![no_main]
#![feature(sync_unsafe_cell)]

use core::{arch, cell, mem, ptr};

use alloc::slice;

#[macro_use]
extern crate alloc;

mod mm;
mod sm;
mod util;

#[allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals
)]
mod multiboot {
    include!(concat!(env!("OUT_DIR"), "/multiboot.rs"));
}

#[cfg(target_arch = "x86")]
arch::global_asm!(include_str!("x86.S"), options(att_syntax));
#[cfg(target_arch = "x86_64")]
arch::global_asm!(include_str!("x86_64.S"), options(att_syntax));

#[no_mangle]
fn main(_multiboot_magic: u32, multiboot_info: u32) -> ! {
    let multiboot_info =
        unsafe { &*((multiboot_info as usize) as *const multiboot::multiboot_info) };

    init_phys_mem_bare();
    init_phys_mem_e820(unsafe {
        slice::from_raw_parts(
            (multiboot_info.mmap_addr as usize) as *const multiboot::multiboot_mmap_entry,
            multiboot_info.mmap_length as usize / size_of::<multiboot::multiboot_mmap_entry>(),
        )
    });

    loop {}
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

fn init_phys_mem_bare() {
    static PHYS_MEM: cell::SyncUnsafeCell<[usize; 2048 / usize::BITS as usize]> =
        cell::SyncUnsafeCell::new([0; 2048 / usize::BITS as usize]);

    let mut phys_mem = mm::PHYS_MEM.lock();
    *phys_mem = mm::PhysicalMemory::new(
        util::bitmap::Bitmap::new(unsafe {
            mem::transmute(ptr::slice_from_raw_parts(
                PHYS_MEM.get(),
                2048 / usize::BITS as usize,
            ))
        }),
        2048,
    );
    phys_mem.mark_used(0, 1024); // system & kernel
}

fn init_phys_mem_e820(phys_mem_map: &[multiboot::multiboot_mmap_entry]) {
    let phys_mem_max: usize = phys_mem_map
        .iter()
        .filter(|phys_mem_entry| phys_mem_entry.type_ == multiboot::MULTIBOOT_MEMORY_AVAILABLE)
        .map(|phys_mem_entry| {
            ((phys_mem_entry.addr + phys_mem_entry.len) / mm::pg::GRANULARITY as u64) as usize
        })
        .max()
        .unwrap();
    let phys_mem_new = mm::PhysicalMemory::new(
        util::bitmap::Bitmap::new(
            vec![usize::MAX; phys_mem_max.div_ceil(usize::BITS as usize)].into_boxed_slice(),
        ),
        0,
    );

    let mut phys_mem = mm::PHYS_MEM.lock();
    *phys_mem = phys_mem_new;
    for phys_mem_entry in phys_mem_map {
        if phys_mem_entry.type_ != multiboot::MULTIBOOT_MEMORY_AVAILABLE {
            continue;
        }

        let frame_start = phys_mem_entry.addr / mm::pg::GRANULARITY as u64;
        let frame_end = frame_start + (phys_mem_entry.len / mm::pg::GRANULARITY as u64);
        let frames = frame_end - frame_start;
        if frames == 0 {
            continue;
        }

        phys_mem.mark_free(frame_start as usize, frames as usize);
    }
    phys_mem.mark_used(0, 1024); // system & kernel
}
