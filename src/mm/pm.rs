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

use core::{mem, ptr};

use crate::util::bitmap::Bitmap;

pub struct PhysicalMemory {
    used: mem::ManuallyDrop<Bitmap>,
    free: usize,
}

impl PhysicalMemory {
    pub(super) const fn empty() -> Self {
        Self {
            used: mem::ManuallyDrop::new(unsafe {
                mem::transmute(ptr::slice_from_raw_parts(
                    ptr::NonNull::<[usize; 0]>::dangling().as_ptr() as *const _,
                    0,
                ))
            }),
            free: 0,
        }
    }

    pub fn new(used: Bitmap, free: usize) -> Self {
        Self {
            used: mem::ManuallyDrop::new(used),
            free,
        }
    }

    pub fn mark_used(&mut self, frame_start: usize, frames: usize) {
        self.used.set_ones(frame_start..frame_start + frames);
        self.free -= frames; // TODO: count 0's
    }

    pub fn mark_free(&mut self, frame_start: usize, frames: usize) {
        self.used.set_zeros(frame_start..frame_start + frames);
        self.free += frames; // TODO: count 1's
    }

    pub fn find_free(&mut self, frames: usize) -> Option<usize> {
        if self.free < frames {
            return None;
        }

        self.used
            .consecutive_zeros(frames)
            .next()
            .map(|frame_range| frame_range.start)
    }
}
