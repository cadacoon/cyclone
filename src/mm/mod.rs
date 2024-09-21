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

static VIRT_MEM: Mutex<VirtualMemory> = Mutex::new(VirtualMemory::bootstrap());
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

struct VirtualMemory {
    page_table_phys_page: usize,
}

impl VirtualMemory {
    const fn bootstrap() -> Self {
        Self {
            page_table_phys_page: 0,
        }
    }

    fn new() -> Self {
        let (page_table_phys_page, page_table_virt_page) =
            VIRT_MEM.lock().unwrap().allocate_contiguous(0x3FF).unwrap();
        let page_table = unsafe { &mut *(page_table_virt_page as *mut PageTable) };

        // Last page is a self-reference
        page_table.0[0x3FF].map(page_table_phys_page);

        Self {
            page_table_phys_page,
        }
    }

    fn allocate_contiguous(&mut self, count: usize) -> Option<(usize, usize)> {
        let mut phys_mem = PHYS_MEM.lock().unwrap();

        // 1. Get contiguous free block of physical memory
        let phys_page = phys_mem.used.consecutive_zeros(count).next()?;

        // 2. Get and commit contiguous free block of virtual memory
        let virt_page_start = 0usize;

        // 3. Commit physical memory
        let phys_page_start = phys_page.start;
        let phys_page_range = phys_page_start..phys_page_start + count;
        phys_mem.used.set_ones(phys_page_range.clone());

        // 4. Write page table
        for (virt_page, phys_page) in
            (virt_page_start..virt_page_start + count).zip(phys_page_range)
        {
            let ptl0_index = virt_page >> 10;
            let ptl0 = &mut PageTable::ptl0().0[ptl0_index];
            if !ptl0.present() {
                let ptl1_phys_page =
                    unsafe { phys_mem.used.consecutive_zeros(1).next().unwrap_unchecked() }.start;
                phys_mem.used.set_ones(ptl1_phys_page..=ptl1_phys_page);
                PageTable::ptl1(0x3FF).0[ptl0_index].map(ptl1_phys_page);

                ptl0.map(ptl1_phys_page);
            }

            let ptl1_index = virt_page & 0x3FF;
            PageTable::ptl1(ptl0_index).0[ptl1_index].map(phys_page);
        }

        Some((virt_page_start, phys_page_start))
    }

    fn allocate(&mut self, length: usize) -> Option<(usize)> {
        self.allocate_contiguous(length)
            .map(|(virt_page_start, _)| virt_page_start)
    }
}

#[repr(transparent)]
struct PageTable([PageTableEntry; 1024]);

impl PageTable {
    fn ptl0() -> &'static mut Self {
        // Last page is a self-reference
        unsafe { &mut *(0xFFC0_0000 as *mut PageTable) }
    }

    fn ptl1(ptl0_index: usize) -> &'static mut Self {
        // Last page is a self-reference
        unsafe { &mut *((0xFFC0_0000 | (ptl0_index << 12)) as *mut PageTable) }
    }
}

#[repr(transparent)]
struct PageTableEntry(u32);

impl PageTableEntry {
    fn present(&self) -> bool {
        false
    }

    fn map(&mut self, phys_page: usize) {}
}
