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

use core::{cell, mem, ptr};

use bitmap::Bitmap;
use spin::Mutex;

use crate::mm::pg::PAGES_PER_TABLE;

use super::pg;

pub struct PhysicalMemory {
    used: Bitmap,
    free: usize,
}

impl PhysicalMemory {
    pub const fn new(used: Bitmap, free: usize) -> Self {
        Self { used, free }
    }

    pub fn mark_used(&mut self, frame_start: usize, count: usize) {
        self.used.set_ones(frame_start..frame_start + count);
        self.free -= count;
    }

    pub fn mark_free(&mut self, frame_start: usize, count: usize) {
        self.used.set_zeros(frame_start..frame_start + count);
        self.free += count;
    }

    pub fn find_free(&mut self, count: usize) -> Option<usize> {
        if self.free < count {
            return None;
        }

        self.used
            .consecutive_zeros(count)
            .next()
            .map(|frame_range| frame_range.start)
    }
}

pub static PHYS_MEM: Mutex<PhysicalMemory> = Mutex::new(PhysicalMemory::new(
    Bitmap::new(unsafe {
        mem::transmute(ptr::slice_from_raw_parts(
            ptr::NonNull::<[usize; 0]>::dangling().as_ptr() as *const _,
            0,
        ))
    }),
    0,
));

const PHYS_MEM_BARE_SIZE: usize = 2048;

pub fn init_phys_mem_bare() {
    static PHYS_MEM_DATA: cell::SyncUnsafeCell<[usize; PHYS_MEM_BARE_SIZE / usize::BITS as usize]> =
        cell::SyncUnsafeCell::new([0; PHYS_MEM_BARE_SIZE / usize::BITS as usize]);

    let mut phys_mem = PHYS_MEM.lock();
    *phys_mem = PhysicalMemory::new(
        Bitmap::new(unsafe {
            mem::transmute(ptr::slice_from_raw_parts(
                PHYS_MEM_DATA.get(),
                PHYS_MEM_BARE_SIZE / usize::BITS as usize,
            ))
        }),
        PHYS_MEM_BARE_SIZE,
    );
    phys_mem.mark_used(0, PAGES_PER_TABLE);
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
    let phys_mem_used =
        vec![usize::MAX; phys_mem_max.div_ceil(usize::BITS as usize)].into_boxed_slice();

    let mut phys_mem = PHYS_MEM.lock();
    phys_mem.used.update(phys_mem_used);

    for phys_mem_entry in phys_mem_map {
        if phys_mem_entry.type_ != multiboot::MULTIBOOT_MEMORY_AVAILABLE {
            continue;
        }

        let mut frame_start = phys_mem_entry.addr / pg::BYTES_PER_PAGE as u64;
        let mut frame_end = frame_start + (phys_mem_entry.len / pg::BYTES_PER_PAGE as u64);

        // already accounted for in init_phys_mem_bare
        frame_start = frame_start.max(PHYS_MEM_BARE_SIZE as u64);
        frame_end = frame_end.max(PHYS_MEM_BARE_SIZE as u64);

        let frames = frame_end - frame_start;
        if frames == 0 {
            continue;
        }

        phys_mem.mark_free(frame_start as usize, frames as usize);
    }
}
