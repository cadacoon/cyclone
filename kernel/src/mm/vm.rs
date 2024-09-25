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
    PHYS_MEM,
};

#[global_allocator]
pub static VIRT_MEM: VirtualMemory = VirtualMemory;

#[derive(Clone)]
pub struct VirtualMemory;

impl VirtualMemory {
    /// Maps frames to free pages
    pub fn map(&self, frame_start: usize, frames: usize) -> Option<Page> {
        let page_start = self.find_free(frames)?;
        for (page, frame) in
            (page_start.0..page_start.0 + frames).zip(frame_start..frame_start + frames)
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

    /// Allocates free frames and maps them to free pages
    pub fn allocate(&self, pages: usize) -> Option<Page> {
        self.allocate_contiguous(pages)
            .map(|(page_start, _)| page_start)
    }

    /// Allocates free frames and maps them to free pages
    pub fn allocate_contiguous(&self, pages: usize) -> Option<(Page, usize)> {
        let frame_start;
        {
            let mut phys_mem = PHYS_MEM.lock();
            frame_start = phys_mem.find_free(pages)?;
            phys_mem.mark_used(frame_start, pages);
        }
        let page_start = self.map(frame_start, pages)?;

        Some((page_start, frame_start))
    }

    /// Frees pages and frames
    pub fn free(&self, page_start: Page, pages: usize) {
        let mut phys_mem = PHYS_MEM.lock();
        for page in page_start.0..page_start.0 + pages {
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

    /// Finds free pages
    fn find_free(&self, pages: usize) -> Option<Page> {
        let mut page_start = 1;
        let mut consecutive_pages = 0;
        while consecutive_pages < pages {
            // not enough remaining pages
            if page_start + pages > PAGES_TOTAL {
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
        self.allocate(pages)
            .map_or(ptr::null_mut(), |page_start| page_start.addr() as *mut u8)
    }

    unsafe fn dealloc(&self, virt_addr: *mut u8, layout: alloc::Layout) {
        let page_start = Page(virt_addr as usize / BYTES_PER_PAGE);
        let pages = layout.size().div_ceil(BYTES_PER_PAGE);
        self.free(page_start, pages);
    }
}

impl acpi::AcpiHandler for VirtualMemory {
    unsafe fn map_physical_region<T>(
        &self,
        phys_addr: usize,
        size: usize,
    ) -> acpi::PhysicalMapping<Self, T> {
        let frame_start = phys_addr / BYTES_PER_PAGE;
        let frames = size.div_ceil(BYTES_PER_PAGE);
        let offset = phys_addr % BYTES_PER_PAGE;
        let virt_addr = self.map(frame_start, frames).unwrap().addr().add(offset);
        acpi::PhysicalMapping::new(
            phys_addr,
            ptr::NonNull::new_unchecked(virt_addr as *mut T),
            size,
            size,
            Self,
        )
    }

    fn unmap_physical_region<T>(_region: &acpi::PhysicalMapping<Self, T>) {}
}
