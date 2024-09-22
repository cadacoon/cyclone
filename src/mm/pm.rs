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

use crate::util::bitmap::Bitmap;

pub struct PhysicalMemory {
    used: Bitmap,
    pub free: usize,
}

impl PhysicalMemory {
    pub(super) const fn new() -> Self {
        Self {
            used: Bitmap::new(),
            free: 1024 * 1024,
        }
    }

    pub fn mark_used(&mut self, page_start: usize, pages: usize) {
        self.used.set_ones(page_start..page_start + pages);
        self.free -= pages;
    }

    pub fn mark_free(&mut self, page_start: usize, pages: usize) {
        self.used.set_zeros(page_start..page_start + pages);
        self.free += pages;
    }

    pub fn find_free(&mut self, pages: usize) -> Option<usize> {
        if self.free < pages {
            return None;
        }

        self.used
            .consecutive_zeros(pages)
            .next()
            .map(|page_range| page_range.start)
    }
}
