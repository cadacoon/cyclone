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

static PHYSICAL_MEMORY: Mutex<PhysicalMemory> = Mutex::new(PhysicalMemory::new());

struct PhysicalMemory {
    used: Bitmap,
}

impl PhysicalMemory {
    const fn new() -> Self {
        Self {
            used: Bitmap::empty(),
        }
    }

    fn allocate_contiguous(&mut self, length: usize) -> Option<usize> {
        let block = self.used.find_zeros(length)?;
        self.used.set_ones(block.start..block.start + length);
        Some(block.start)
    }

    fn free(&mut self, address: usize, length: usize) {
        self.used.set_zeros(address..address + length);
    }
}

struct AddressSpace {}

impl AddressSpace {
    fn allocate_contiguous(&mut self, length: usize) -> Option<(usize, usize)> {
        let mut phys_mem = PHYSICAL_MEMORY.lock().unwrap();

        // 1. Get contiguous free block of physical memory
        let phys_addr = phys_mem.allocate_contiguous(length)?;

        // 2. Get contiguous free block of virtual memory
        let virt_addr = 0usize;

        // 3. Write page table
        // for virt_addr in virt_addr..virt_addr + length {
        //     let ptl0 = virt_addr >> (10 + 12);
        //     let ptl1_present = false;
        //     if !ptl1_present {
        //         let ptl1 = phys_mem.allocate_contiguous(1)?;
        //         page_table[ptl0] = ptl1;
        //     }

        //     let ptl1 = page_table[ptl0];
        //     ptl1[virt_addr >> 12] = phys_addr + 0;
        // }

        Some((phys_addr, virt_addr))
    }

    fn allocate(&mut self, length: usize) -> Option<usize> {
        let mut phys_mem = PHYSICAL_MEMORY.lock().unwrap();

        // 1. Get free blocks of physical memory
        let phys_addrs = [0usize];

        // 2. Get contiguous free block of virtual memory
        let virt_addr = 0usize;

        // 3. Write page table
        // for virt_addr in virt_addr..virt_addr + length {
        //     let ptl0 = virt_addr >> (10 + 12);
        //     let ptl1_present = false;
        //     if !ptl1_present {
        //         let ptl1 = phys_mem.allocate_contiguous(1)?;
        //         page_table[ptl0] = ptl1;
        //     }

        //     let ptl1 = page_table[ptl0];
        //     ptl1[virt_addr >> 12] = phys_addrs[0];
        // }

        None
    }
}
