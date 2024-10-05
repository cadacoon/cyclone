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

use core::{alloc, ptr};

use super::{
    pg::{Page, BYTES_PER_PAGE, PAGES_PER_TABLE, PAGES_TOTAL, PAGE_TABLE},
    KERNEL_VMA, PHYS_MEM,
};

#[global_allocator]
pub static VIRT_MEM: VirtualMemory = VirtualMemory;

pub struct VirtualMemory;

impl VirtualMemory {
    pub fn map(&self, page_start: Page, frame_start: usize, count: usize) -> Option<Page> {
        let page_start = self.find_free(page_start, count)?;
        for (page, frame) in
            (page_start.0..page_start.0 + count).zip(frame_start..frame_start + count)
        {
            let page = Page(page);
            let page_table = unsafe { &mut *PAGE_TABLE };
            #[cfg(target_arch = "x86_64")]
            let page_table = page_table.table_create(page);
            #[cfg(target_arch = "x86_64")]
            let page_table = page_table.table_create(page);
            let page_table = page_table.table_create(page);
            let page_table_entry = &mut page_table[page];
            if page_table_entry.used() {
                panic!("non-contiguous");
            }

            page_table_entry.map(frame);
        }

        Some(page_start)
    }

    pub fn allocate(&self, page_start: Page, count: usize) -> Option<Page> {
        self.allocate_contiguous(page_start, count)
            .map(|(page_start, _)| page_start)
    }

    pub fn allocate_contiguous(&self, page_start: Page, count: usize) -> Option<(Page, usize)> {
        let frame_start;
        {
            let mut phys_mem = PHYS_MEM.lock();
            frame_start = phys_mem.find_free(count)?;
            phys_mem.mark_used(frame_start, count);
        }
        let page_start = self.map(page_start, frame_start, count)?;

        Some((page_start, frame_start))
    }

    pub fn free(&self, page_start: Page, count: usize) {
        let mut phys_mem = PHYS_MEM.lock();
        for page in page_start.0..page_start.0 + count {
            let page = Page(page);
            let page_table = unsafe { &mut *PAGE_TABLE };
            #[cfg(target_arch = "x86_64")]
            let page_table = page_table.table(page).expect("already freed");
            #[cfg(target_arch = "x86_64")]
            let page_table = page_table.table(page).expect("already freed");
            let page_table = page_table.table(page).expect("already freed");
            let page_table_entry = &mut page_table[page];
            if !page_table_entry.used() {
                panic!("already freed")
            }

            let frame = page_table_entry.unmap();
            phys_mem.mark_free(frame, 1);
        }
    }

    fn find_free(&self, page_start: Page, count: usize) -> Option<Page> {
        let mut page_start = page_start.0;
        let mut consecutive_pages = 0;
        while consecutive_pages < count {
            // not enough remaining pages
            if page_start + count > PAGES_TOTAL {
                return None;
            }

            let page = Page(page_start + consecutive_pages);
            let page_table = unsafe { &mut *PAGE_TABLE };
            #[cfg(target_arch = "x86_64")]
            let Some(page_table) = page_table.table(page) else {
                consecutive_pages += PAGES_PER_TABLE * PAGES_PER_TABLE * PAGES_PER_TABLE;
                continue;
            };
            #[cfg(target_arch = "x86_64")]
            let Some(page_table) = page_table.table(page) else {
                consecutive_pages += PAGES_PER_TABLE * PAGES_PER_TABLE;
                continue;
            };
            let Some(page_table) = page_table.table(page) else {
                consecutive_pages += PAGES_PER_TABLE;
                continue;
            };
            if !page_table[page].used() {
                consecutive_pages += 1;
                continue;
            }

            page_start += 1 + consecutive_pages;
            consecutive_pages = 0;
        }

        Some(Page(page_start))
    }
}

unsafe impl alloc::GlobalAlloc for VirtualMemory {
    unsafe fn alloc(&self, layout: alloc::Layout) -> *mut u8 {
        let pages = layout.size().div_ceil(BYTES_PER_PAGE);
        self.allocate(
            Page(((&KERNEL_VMA as *const u8 as usize) / BYTES_PER_PAGE) & PAGES_TOTAL),
            pages,
        )
        .map_or(ptr::null_mut(), |page_start| page_start.ptr() as *mut u8)
    }

    unsafe fn dealloc(&self, virt_addr: *mut u8, layout: alloc::Layout) {
        let page_start = Page(virt_addr as usize / BYTES_PER_PAGE);
        let pages = layout.size().div_ceil(BYTES_PER_PAGE);
        self.free(page_start, pages);
    }
}
