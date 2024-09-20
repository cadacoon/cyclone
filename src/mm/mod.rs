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

use std::sync::Mutex;

use crate::util::bitmap::Bitmap;

static PHYS_MEM: Mutex<PhysicalMemory> = Mutex::new(PhysicalMemory::new());

struct PhysicalMemory {
    available: usize,
    used: Bitmap,
}

impl PhysicalMemory {
    const fn new() -> Self {
        Self {
            available: 0,
            used: Bitmap::empty(),
        }
    }
}

struct AddressSpace {}

impl AddressSpace {
    fn allocate_contiguous(&mut self, length: usize) -> Option<(usize, usize)> {
        let mut phys_mem = PHYS_MEM.lock().unwrap();

        // 1. Get contiguous free block of physical memory
        let phys_mem_block = phys_mem.used.consecutive_zeros(length).next()?;

        // 2. Get and commit contiguous free block of virtual memory
        let virt_addr = 0usize;

        // 3. Commit physical memory
        let phys_addr_start = phys_mem_block.start;
        let phys_mem_block = phys_addr_start..phys_addr_start + length;
        phys_mem.used.set_ones(phys_mem_block.clone());

        // 3. Write page table
        for (virt_addr, phys_addr) in (virt_addr..virt_addr + length).zip(phys_mem_block) {
            let ptl0_index = virt_addr >> 10;
            let ptl1_present = true;
            if !ptl1_present {
                let ptl1_phys_addr =
                    unsafe { phys_mem.used.consecutive_zeros(1).next().unwrap_unchecked() }.start;
                phys_mem.used.set_ones(ptl1_phys_addr..=ptl1_phys_addr);

                page_table[ptl0_index] = ptl1_phys_addr;
            }

            let ptl1_index = virt_addr & (1 << 10);
            page_table[ptl0_index][ptl1_index] = phys_addr
        }

        Some((phys_addr_start, virt_addr))
    }

    fn allocate(&mut self, length: usize) -> Option<usize> {
        let mut phys_mem = PHYS_MEM.lock().unwrap();

        // 1. Get free blocks of physical memory
        let mut phys_mem_blocks = phys_mem.used.consecutive_zeros(1);

        // 2. Get and commit contiguous free block of virtual memory
        let virt_addr_start = 0usize;

        // 3. Commit physical memory and write page table
        let mut phys_mem_block = 0..0;
        for virt_addr in virt_addr_start..virt_addr_start + length {
            let phys_addr = match phys_mem_block.next() {
                Some(phys_addr) => phys_addr,
                None => {
                    phys_mem_block = unsafe { phys_mem_blocks.next().unwrap_unchecked() };
                    let phys_addr = phys_mem_block.start;
                    let remaining = length - (virt_addr - virt_addr_start);
                    phys_mem_block = phys_addr..phys_addr + phys_mem_block.len().min(remaining);
                    phys_mem_blocks.set_ones(phys_mem_block.clone());
                    unsafe { phys_mem_block.next().unwrap_unchecked() }
                }
            };

            let ptl0_index = virt_addr >> 10;
            let ptl1_present = true;
            if !ptl1_present {
                let ptl1_phys_addr =
                    unsafe { phys_mem.used.consecutive_zeros(1).next().unwrap_unchecked() }.start;
                phys_mem.used.set_ones(ptl1_phys_addr..=ptl1_phys_addr);

                page_table[ptl0_index] = ptl1_phys_addr;
            }

            let ptl1_index = virt_addr & (1 << 10);
            page_table[ptl0_index][ptl1_index] = phys_addr
        }

        None
    }
}
