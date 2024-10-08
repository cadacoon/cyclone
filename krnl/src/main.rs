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
#![feature(abi_x86_interrupt, naked_functions, sync_unsafe_cell)]

use core::{arch, hint, panic, slice};

use log::error;

#[macro_use]
extern crate alloc;

mod ex;
mod mm;
mod tty;

#[cfg(target_arch = "x86")]
arch::global_asm!(include_str!("x86.S"));
#[cfg(target_arch = "x86_64")]
arch::global_asm!(include_str!("x86_64.S"));

#[no_mangle]
extern "C" fn main(multiboot_magic: u32, multiboot_info: u32) -> ! {
    if multiboot_magic != multiboot::MULTIBOOT_BOOTLOADER_MAGIC {
        loop {
            hint::spin_loop();
        }
    }
    let multiboot_info = unsafe {
        &*((multiboot_info as usize + (&mm::KERNEL_VMA as *const u8 as usize))
            as *const multiboot::multiboot_info)
    };
    if multiboot_info.flags & multiboot::MULTIBOOT_INFO_MEM_MAP == 0 {
        loop {
            hint::spin_loop();
        }
    }

    mm::init_virt_mem();
    mm::init_phys_mem();

    tty::init();
    mm::init_phys_mem_e820(unsafe {
        slice::from_raw_parts(
            (multiboot_info.mmap_addr as usize + (&mm::KERNEL_VMA as *const u8 as usize))
                as *const multiboot::multiboot_mmap_entry,
            multiboot_info.mmap_length as usize / size_of::<multiboot::multiboot_mmap_entry>(),
        )
    });

    ex::run();
}

#[no_mangle]
extern "C" fn main_ap() {
    ex::run();
}

#[panic_handler]
fn panic(info: &panic::PanicInfo) -> ! {
    error!("{}", info.message());

    loop {
        hint::spin_loop();
    }
}
