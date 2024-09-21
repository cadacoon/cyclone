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

#![no_std]
#![no_main]
#![feature(sync_unsafe_cell)]

use core::slice;

use multiboot::multiboot_mmap_entry;

extern crate alloc;

mod bs;
pub mod mm;
pub mod sm;
pub mod util;

#[no_mangle]
unsafe fn main(_multiboot_magic: u32, multiboot_info: &mut multiboot::multiboot_info) -> ! {
    // 1. init physical memory
    {
        let mut phys_mem = mm::PHYS_MEM.lock();
        phys_mem.mark_used(0, 1024 * 1024);
        for mmap_entry in slice::from_raw_parts(
            multiboot_info.mmap_addr as usize as *const multiboot_mmap_entry,
            multiboot_info.mmap_length as usize / size_of::<multiboot_mmap_entry>(),
        ) {
            if mmap_entry.type_ != multiboot::MULTIBOOT_MEMORY_AVAILABLE {
                continue;
            }
            phys_mem.mark_free(
                (mmap_entry.addr >> 12) as usize,
                (mmap_entry.len >> 12) as usize,
            );
        }
        phys_mem.mark_used(0, 1024);
    }

    loop {}
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals
)]
mod multiboot {
    include!(concat!(env!("OUT_DIR"), "/multiboot.rs"));
}
