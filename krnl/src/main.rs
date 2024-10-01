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
#![feature(abi_x86_interrupt, sync_unsafe_cell, negative_impls)]

use core::{arch, hint, panic, slice};

use tracing::{error, info};

#[macro_use]
extern crate alloc;

mod int;
mod mm;
mod sm;
mod tty;
mod util;

#[cfg(target_arch = "x86")]
arch::global_asm!(include_str!("x86.S"));
#[cfg(target_arch = "x86_64")]
arch::global_asm!(include_str!("x86_64.S"));

#[no_mangle]
fn main(multiboot_magic: u32, multiboot_info: u32) -> ! {
    mm::sm::init();
    int::init();

    if multiboot_magic != multiboot::MULTIBOOT_BOOTLOADER_MAGIC {
        halt();
    }
    let multiboot_info = unsafe {
        &*((multiboot_info as usize + (&mm::KERNEL_VMA as *const u8 as usize))
            as *const multiboot::multiboot_info)
    };
    if multiboot_info.flags & multiboot::MULTIBOOT_INFO_MEM_MAP == 0 {
        halt();
    }

    mm::init_virt_mem();
    mm::init_phys_mem_bare();
    mm::init_phys_mem_e820(unsafe {
        slice::from_raw_parts(
            (multiboot_info.mmap_addr as usize + (&mm::KERNEL_VMA as *const u8 as usize))
                as *const multiboot::multiboot_mmap_entry,
            multiboot_info.mmap_length as usize / size_of::<multiboot::multiboot_mmap_entry>(),
        )
    });

    tty::init_logging();

    info!("Meerkat Operating System {}", env!("CARGO_PKG_VERSION"));

    halt();
}

#[no_mangle]
fn main_ap() -> ! {
    halt()
}

#[panic_handler]
fn panic(info: &panic::PanicInfo) -> ! {
    error!("{}", info.message());

    halt();
}

#[inline(always)]
fn halt() -> ! {
    loop {
        hint::spin_loop();
    }
}
