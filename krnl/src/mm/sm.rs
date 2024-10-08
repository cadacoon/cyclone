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

use core::{arch, cell, mem, ptr};

const DESCRIPTOR_NULL: usize = 0;
const DESCRIPTOR_KCODE: usize = 1;
const DESCRIPTOR_KDATA: usize = 2;
const DESCRIPTOR_UCODE: usize = 3;
const DESCRIPTOR_UDATA: usize = 4;
const DESCRIPTOR_TSS: usize = 5;
#[cfg(target_arch = "x86")]
const DESCRIPTOR_GS: usize = 6;
#[cfg(target_arch = "x86_64")]
const DESCRIPTOR_TSS64: usize = 6;

#[no_mangle]
static DESCRIPTOR_TABLE: cell::SyncUnsafeCell<[Descriptor; 7]> = cell::SyncUnsafeCell::new([
    // NULL
    unsafe { Descriptor::zeroed() },
    // KCODE
    Descriptor::new(
        0x00000000,
        0xFFFFF,
        DescriptorAccess::A
            .union(DescriptorAccess::RW)
            .union(DescriptorAccess::E)
            .union(DescriptorAccess::S)
            .union(DescriptorAccess::P),
        0,
        #[cfg(target_arch = "x86")]
        DescriptorFlags::DB.union(DescriptorFlags::G),
        #[cfg(target_arch = "x86_64")]
        DescriptorFlags::L.union(DescriptorFlags::G),
    ),
    // KDATA
    Descriptor::new(
        0x00000000,
        0xFFFFF,
        DescriptorAccess::A
            .union(DescriptorAccess::RW)
            .union(DescriptorAccess::S)
            .union(DescriptorAccess::P),
        0,
        DescriptorFlags::DB.union(DescriptorFlags::G),
    ),
    // UCODE
    Descriptor::new(
        0x00000000,
        0xFFFFF,
        DescriptorAccess::A
            .union(DescriptorAccess::RW)
            .union(DescriptorAccess::E)
            .union(DescriptorAccess::S)
            .union(DescriptorAccess::P),
        3,
        #[cfg(target_arch = "x86")]
        DescriptorFlags::DB.union(DescriptorFlags::G),
        #[cfg(target_arch = "x86_64")]
        DescriptorFlags::L.union(DescriptorFlags::G),
    ),
    // UDATA
    Descriptor::new(
        0x00000000,
        0xFFFFF,
        DescriptorAccess::A
            .union(DescriptorAccess::RW)
            .union(DescriptorAccess::S)
            .union(DescriptorAccess::P),
        3,
        DescriptorFlags::DB.union(DescriptorFlags::G),
    ),
    // TSS
    unsafe { Descriptor::zeroed() },
    // TSS64 / GS
    unsafe { Descriptor::zeroed() },
]);

#[repr(C, packed(2))]
struct DescriptorTableRegister {
    size: u16,
    offset: *mut [Descriptor],
}

#[repr(C)]
struct Descriptor {
    limit_0_15: u16,
    base_0_15: u16,
    base_16_23: u8,
    access: u8,
    flags_and_limit_16_19: u8,
    base_24_31: u8,
}

impl Descriptor {
    const unsafe fn zeroed() -> Self {
        mem::MaybeUninit::zeroed().assume_init()
    }

    const fn new(
        base: u32,
        limit: u32,
        access: DescriptorAccess,
        dpl: u8,
        flags: DescriptorFlags,
    ) -> Self {
        Self {
            limit_0_15: limit as u16,
            base_0_15: base as u16,
            base_16_23: (base >> 16) as u8,
            access: access.bits() | dpl << 5,
            flags_and_limit_16_19: (limit >> 16) as u8 | flags.bits(),
            base_24_31: (base >> 24) as u8,
        }
    }
}

bitflags::bitflags! {
    struct DescriptorAccess: u8 {
        const A = 1 << 0;
        const RW = 1 << 1;
        const DC = 1 << 2;
        const E = 1 << 3;
        const S = 1 << 4;
        const P = 1 << 7;
    }

    #[derive(Copy, Clone, Debug)]
    struct DescriptorFlags: u8 {
        const L = 1 << 5;
        const DB = 1 << 6;
        const G = 1 << 7;
    }
}

#[cfg(target_arch = "x86")]
#[repr(C)]
#[derive(Default)]
pub struct TaskStateSegment {
    link: u16,
    _reserved_0: u16,
    esp0: u32,
    ss0: u16,
    _reserved_1: u16,
    esp1: u32,
    ss1: u16,
    _reserved_2: u16,
    esp2: u32,
    ss2: u16,
    _reserved_3: u16,
    cr3: u32,
    eip: u32,
    eflags: u32,
    eax: u32,
    ecx: u32,
    edx: u32,
    ebx: u32,
    esp: u32,
    ebp: u32,
    esi: u32,
    edi: u32,
    es: u16,
    _reserved_4: u16,
    cs: u16,
    _reserved_5: u16,
    ss: u16,
    _reserved_6: u16,
    ds: u16,
    _reserved_7: u16,
    fs: u16,
    _reserved_8: u16,
    gs: u16,
    _reserved_9: u16,
    ldtr: u16,
    _reserved_10: u16,
    _reserved_11: u16,
    iopb: u16,
}

#[cfg(target_arch = "x86_64")]
#[repr(C, packed(4))]
#[derive(Default)]
pub struct TaskStateSegment {
    _reserved_0: u32,
    privilege_stack_table: [u64; 3],
    _reserved_1: u64,
    interrupt_stack_table: [u64; 7],
    _reserved_2: u64,
    _reserved_3: u16,
    iopb: u16,
}

impl TaskStateSegment {
    pub unsafe fn load(&self) {
        let base = ptr::addr_of!(self) as usize;
        let limit = size_of_val(self);
        (&mut *DESCRIPTOR_TABLE.get())[DESCRIPTOR_TSS] = Descriptor::new(
            base as u32,
            limit as u32,
            DescriptorAccess::A
                .union(DescriptorAccess::E)
                .union(DescriptorAccess::P),
            0,
            DescriptorFlags::empty(),
        );
        #[cfg(target_arch = "x86_64")]
        {
            (&mut *DESCRIPTOR_TABLE.get())[DESCRIPTOR_TSS64].limit_0_15 = (base >> 32) as u16;
            (&mut *DESCRIPTOR_TABLE.get())[DESCRIPTOR_TSS64].base_0_15 = (base >> 48) as u16;
        }
        {
            arch::asm!("ltr {0:x}", in(reg) DESCRIPTOR_TSS << 3, options(nostack, preserves_flags))
        }
    }
}

pub struct GS;

impl GS {
    #[cfg(target_arch = "x86")]
    pub unsafe fn set(base: usize, limit: usize) {
        (&mut *DESCRIPTOR_TABLE.get())[DESCRIPTOR_GS] = Descriptor::new(
            base as u32,
            limit as u32,
            DescriptorAccess::A
                .union(DescriptorAccess::RW)
                .union(DescriptorAccess::E)
                .union(DescriptorAccess::S)
                .union(DescriptorAccess::P),
            0,
            DescriptorFlags::DB.union(DescriptorFlags::G),
        );
        arch::asm!("mov gs, {0:x}", in(reg) DESCRIPTOR_GS << 3);
    }

    #[cfg(target_arch = "x86_64")]
    pub unsafe fn set(base: usize, _limit: usize) {
        arch::asm!(
            "wrmsr",
            in("ecx") 0xC0000101u32,
            in("eax") base as u32,
            in("edx") (base >> 32) as u32,
        );
    }
}
