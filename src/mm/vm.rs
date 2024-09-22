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

use core::{alloc, arch::asm, mem::MaybeUninit, ops, ptr};

use super::PHYS_MEM;

pub struct VirtualMemory {
    ptl0_phys_page: usize,
}

impl VirtualMemory {
    pub fn r#use<F: FnOnce()>(&self, f: F) {
        let previous_ptl0_phys_page: usize;
        unsafe {
            asm!(
                "mov {}, cr3",
                "mov cr3, {}",
                out(reg) previous_ptl0_phys_page,
                in(reg) self.ptl0_phys_page,
                options(nostack, preserves_flags)
            );
        }
        f();
        unsafe {
            asm!(
                "mov cr3, {}",
                in(reg) previous_ptl0_phys_page,
                options(nostack, preserves_flags)
            );
        }
    }
}

impl Drop for VirtualMemory {
    fn drop(&mut self) {
        self.r#use(|| todo!());
    }
}

pub struct VirtualMemoryScope;

impl VirtualMemoryScope {
    pub fn allocate(&self, pages: usize) -> Option<usize> {
        self.allocate_contiguous(pages)
            .map(|(page_start, _)| page_start)
    }

    pub fn allocate_contiguous(&self, pages: usize) -> Option<(usize, usize)> {
        let mut phys_mem = PHYS_MEM.lock();

        // 1. get contiguous free block of physical memory
        let phys_page_start = phys_mem.find_free(pages)?;

        // 2. get contiguous free block of virtual memory
        let page_start = self.find_free(pages)?;

        // 3. commit physical memory
        phys_mem.mark_used(phys_page_start, pages);

        // 4. commit virtual memory by writing page table
        for (page, phys_page) in
            (page_start..page_start + pages).zip(phys_page_start..phys_page_start + pages)
        {
            let ptl0_index = page >> 10;
            let ptl0_entry = unsafe { &mut PageTable::ptl0().0[ptl0_index] };
            if ptl0_entry.free() {
                // allocate page table, note that page tables are owned by the address space
                let ptl1_phys_page = phys_mem.find_free(1).unwrap();
                phys_mem.mark_used(ptl1_phys_page, 1);

                ptl0_entry.map(ptl1_phys_page);
            }

            let ptl1_index = page & 0x3FF;
            let ptl1_entry = unsafe { &mut PageTable::ptl1(ptl0_index).0[ptl1_index] };
            /*if !ptl1_entry.free() {
                panic!("non-contiguous {}", ptl1_entry.0);
            }*/

            ptl1_entry.map(phys_page);
        }

        Some((page_start, phys_page_start))
    }

    pub fn free(&self, page_start: usize, pages: usize) {
        let mut phys_mem = PHYS_MEM.lock();

        for page in page_start..page_start + pages {
            let ptl0_index = page >> 10;
            let ptl0_entry = unsafe { &mut PageTable::ptl0().0[ptl0_index] };
            if ptl0_entry.free() {
                panic!("already freed")
            }

            let ptl1_index = page & 0x3FF;
            let ptl1_entry = unsafe { &mut PageTable::ptl1(ptl0_index).0[ptl1_index] };
            if ptl1_entry.free() {
                panic!("already freed")
            }

            let phys_page = ptl1_entry.unmap();
            phys_mem.mark_free(phys_page, 1);
        }
    }

    fn find_free(&self, pages: usize) -> Option<usize> {
        let mut page_start = 1;
        let mut consecutive_pages = 0;
        while consecutive_pages < pages {
            // not enough remaining pages
            if page_start + pages > 0xFFFFF {
                return None;
            }
            let page = page_start + consecutive_pages;

            let ptl0_index = page >> 10;
            let ptl0_entry = unsafe { &mut PageTable::ptl0().0[ptl0_index] };
            if ptl0_entry.free() {
                consecutive_pages += 1024;
                continue;
            }

            let ptl1_index = page & 0x3FF;
            let ptl1_entry = unsafe { &mut PageTable::ptl1(ptl0_index).0[ptl1_index] };
            if !ptl1_entry.free() {
                consecutive_pages += 1;
                continue;
            }

            page_start += consecutive_pages;
            consecutive_pages = 0;
        }
        Some(page_start)
    }
}

unsafe impl alloc::GlobalAlloc for VirtualMemoryScope {
    unsafe fn alloc(&self, layout: alloc::Layout) -> *mut u8 {
        let pages = ((layout.size() - 1) >> 12) + 1;
        self.allocate(pages)
            .map_or(ptr::null_mut(), |page_start| (page_start << 12) as *mut u8)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: alloc::Layout) {
        let page_start = ((ptr as usize - 1) >> 12) + 1;
        let pages = ((layout.size() - 1) >> 12) + 1;
        self.free(page_start, pages);
    }
}

#[repr(C, align(4096))]
pub struct PageTable([PageTableEntry; 1024]);

impl PageTable {
    pub const fn new() -> Self {
        PageTable([PageTableEntry(0); 1024])
    }

    unsafe fn ptl0() -> &'static mut Self {
        // self-reference
        Self::ptl1(0x3FF)
    }

    unsafe fn ptl1(ptl0_index: usize) -> &'static mut Self {
        // self-reference
        &mut *((((0x3FF << 10) | ptl0_index) << 12) as *mut PageTable)
    }
}

impl ops::Index<usize> for PageTable {
    type Output = PageTableEntry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl ops::IndexMut<usize> for PageTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PageTableEntry(u32);

impl PageTableEntry {
    const FREE: u32 = 0;
    const PRESENT: u32 = 1 << 0;
    const WRITEABLE: u32 = 1 << 1;

    #[inline(always)]
    pub fn free(&self) -> bool {
        self.0 == Self::FREE
    }

    #[inline(always)]
    pub fn map(&mut self, phys_page: usize) {
        self.0 = (phys_page << 12) as u32 | Self::PRESENT | Self::WRITEABLE;
    }

    #[inline(always)]
    pub fn unmap(&mut self) -> usize {
        let phys_page = (self.0 >> 12) as usize;
        self.0 = Self::FREE;
        phys_page
    }
}
