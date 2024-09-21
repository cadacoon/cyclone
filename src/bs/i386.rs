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

use core::{arch, cell, hint};

use crate::mm;

pub static BOOT_PTL0: cell::SyncUnsafeCell<mm::PageTable> =
    cell::SyncUnsafeCell::new(mm::PageTable::new());
pub static BOOT_PTL1: cell::SyncUnsafeCell<mm::PageTable> =
    cell::SyncUnsafeCell::new(mm::PageTable::new());

arch::global_asm!(include_str!("i386_entry.S"), options(att_syntax));

// This method should be as barebones as possible and never call any method
// outside in other sections.
// The only reason why this method exists at all is to have minimal assembly.
#[no_mangle]
#[link_section = ".multiboot.init"]
unsafe fn main_bootstrap(multiboot_magic: u32, multiboot_info: usize) -> ! {
    let boot_ptl1_virt_addr = BOOT_PTL1.get();
    let boot_ptl1_phys_addr = boot_ptl1_virt_addr.byte_sub(0xF000_0000);
    let boot_ptl1 = &mut *boot_ptl1_phys_addr;
    for phys_page in 0..1024 {
        boot_ptl1[phys_page].map(phys_page); // identity
    }

    let boot_ptl0_virt_addr = BOOT_PTL0.get();
    let boot_ptl0_phys_addr = boot_ptl0_virt_addr.byte_sub(0xF000_0000);
    let boot_ptl0 = &mut *boot_ptl0_phys_addr;
    boot_ptl0[0x000].map((boot_ptl1_phys_addr as usize) >> 12); // identity
    boot_ptl0[0x3C0].map((boot_ptl1_phys_addr as usize) >> 12);
    boot_ptl0[0x3FF].map((boot_ptl0_phys_addr as usize) >> 12); // self-referential

    // enable paging
    arch::asm!(
        "mov cr3, {}",
        "mov {tmp}, cr0",
        "or {tmp}, 0x80010000",
        "mov cr0, {tmp}",
        in(reg) boot_ptl0_phys_addr,
        tmp = out(reg) _,
    );

    // reinitialize stack (use virtual addresses)
    arch::asm!(
        "mov {tmp}, $stack_top",
        "sub {tmp}, 16",
        "sub {tmp}, 8",
        "mov esp, {tmp}",
        "mov ebp, {tmp}",
        tmp = out(reg) _,
    );

    // call main with multiboot parameters
    arch::asm!(
        "push ebp",
        "mov ebp, esp",
        "push {multiboot_info}",
        "push {multiboot_magic}",
        "lea {tmp}, main",
        "call {tmp}",
        tmp = out(reg) _,
        multiboot_magic = in(reg) multiboot_magic,
        multiboot_info = in(reg) multiboot_info,
    );

    // main is of type never
    hint::unreachable_unchecked();
}
