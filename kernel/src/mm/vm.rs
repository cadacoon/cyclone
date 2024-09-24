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

use super::PHYS_MEM;

#[global_allocator]
pub static VIRT_MEM: VirtualMemory = VirtualMemory;

#[derive(Clone)]
pub struct VirtualMemory;

impl VirtualMemory {
    /// Maps frames to free pages
    pub fn map(&self, frame_start: usize, frames: usize) -> Option<usize> {
        let page_start = self.find_free(frames)?;
        for (page, frame) in
            (page_start..page_start + frames).zip(frame_start..frame_start + frames)
        {
            let page = super::pg::Page(page);
            let page_table = unsafe { &mut *super::pg::PAGE_TABLE };
            let page_table = page_table.table_create(page);
            let page_table_entry = &mut page_table[page];
            if !page_table_entry.free() {
                panic!("non-contiguous");
            }

            page_table_entry.map(frame);
        }

        Some(page_start)
    }

    /// Allocates free frames and maps them to free pages
    pub fn allocate(&self, pages: usize) -> Option<usize> {
        self.allocate_contiguous(pages)
            .map(|(page_start, _)| page_start)
    }

    /// Allocates free frames and maps them to free pages
    pub fn allocate_contiguous(&self, pages: usize) -> Option<(usize, usize)> {
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
    pub fn free(&self, page_start: usize, pages: usize) {
        let mut phys_mem = PHYS_MEM.lock();
        for page in page_start..page_start + pages {
            let page = super::pg::Page(page);
            let page_table = unsafe { &mut *super::pg::PAGE_TABLE };
            let page_table = page_table.table(page).expect("already freed");
            let page_table_entry = &mut page_table[page];
            if page_table_entry.free() {
                panic!("already freed")
            }

            let frame = page_table_entry.unmap();
            phys_mem.mark_free(frame, 1);
        }
    }

    /// Finds free pages
    fn find_free(&self, pages: usize) -> Option<usize> {
        let mut page_start = 1;
        let mut consecutive_pages = 0;
        while consecutive_pages < pages {
            // not enough remaining pages
            if page_start + pages > 0xFFFFF {
                return None;
            }

            let page = super::pg::Page(page_start + consecutive_pages);
            let page_table = unsafe { &mut *super::pg::PAGE_TABLE };
            let Some(page_table) = page_table.table(page) else {
                consecutive_pages += 1024;
                continue;
            };
            if page_table[page].free() {
                consecutive_pages += 1;
                continue;
            }

            page_start += 1 + consecutive_pages;
            consecutive_pages = 0;
        }

        Some(page_start)
    }
}

unsafe impl alloc::GlobalAlloc for VirtualMemory {
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

impl acpi::AcpiHandler for VirtualMemory {
    unsafe fn map_physical_region<T>(
        &self,
        phys_addr: usize,
        size: usize,
    ) -> acpi::PhysicalMapping<Self, T> {
        let offset = phys_addr % super::pg::GRANULARITY;
        let page = self
            .map(
                phys_addr / super::pg::GRANULARITY,
                size.div_ceil(super::pg::GRANULARITY),
            )
            .unwrap();
        let virt_addr = page * super::pg::GRANULARITY + offset;
        acpi::PhysicalMapping::new(
            phys_addr,
            ptr::NonNull::new_unchecked((virt_addr) as *mut T),
            size,
            size,
            Self,
        )
    }

    fn unmap_physical_region<T>(_region: &acpi::PhysicalMapping<Self, T>) {}
}
