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

mod pg;
mod pm;
mod sm;
mod vm;

pub use pm::*;
pub use vm::*;

use core::{cell, mem, ptr};

use crate::bitmap::Bitmap;

extern "C" {
    pub static KERNEL_LMA: u8;
    pub static KERNEL_VMA: u8;
}

pub fn init_virt_mem() {
    (unsafe { &mut *(pg::PAGE_TABLE) })[pg::Page(0)].unmap(); // identity
}

pub fn init_phys_mem_bare() {
    static PHYS_MEM: cell::SyncUnsafeCell<[usize; 2048 / usize::BITS as usize]> =
        cell::SyncUnsafeCell::new([0; 2048 / usize::BITS as usize]);

    let mut phys_mem = pm::PHYS_MEM.lock();
    *phys_mem = PhysicalMemory::new(
        Bitmap::new(unsafe {
            mem::transmute(ptr::slice_from_raw_parts(
                PHYS_MEM.get(),
                2048 / usize::BITS as usize,
            ))
        }),
        2048,
    );
    phys_mem.mark_used(0, 1024); // system & kernel
}

pub fn init_phys_mem_e820(phys_mem_map: &[multiboot::multiboot_mmap_entry]) {
    let phys_mem_max: usize = phys_mem_map
        .iter()
        .filter(|phys_mem_entry| phys_mem_entry.type_ == multiboot::MULTIBOOT_MEMORY_AVAILABLE)
        .map(|phys_mem_entry| {
            ((phys_mem_entry.addr + phys_mem_entry.len) / pg::BYTES_PER_PAGE as u64) as usize
        })
        .max()
        .unwrap();
    let phys_mem_new = PhysicalMemory::new(
        Bitmap::new(
            vec![usize::MAX; phys_mem_max.div_ceil(usize::BITS as usize)].into_boxed_slice(),
        ),
        0,
    );

    let mut phys_mem = PHYS_MEM.lock();
    *phys_mem = phys_mem_new;
    for phys_mem_entry in phys_mem_map {
        if phys_mem_entry.type_ != multiboot::MULTIBOOT_MEMORY_AVAILABLE {
            continue;
        }

        let frame_start = phys_mem_entry.addr / pg::BYTES_PER_PAGE as u64;
        let frame_end = frame_start + (phys_mem_entry.len / pg::BYTES_PER_PAGE as u64);
        let frames = frame_end - frame_start;
        if frames == 0 {
            continue;
        }

        phys_mem.mark_free(frame_start as usize, frames as usize);
    }
    phys_mem.mark_used(0, 1024); // system & kernel
}
