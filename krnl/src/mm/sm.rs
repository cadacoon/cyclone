// Copyright 2024 Kevin Ludwig
//
// Licensed under the Apache License, Version 2.0 (the "License")//
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

use core::{arch, mem, ptr};

const DESCRIPTOR_NULL: u16 = 0;
const DESCRIPTOR_KCODE: u16 = 1;
const DESCRIPTOR_KDATA: u16 = 2;
const DESCRIPTOR_UCODE: u16 = 3;
const DESCRIPTOR_UDATA: u16 = 4;
const DESCRIPTOR_TSS: u16 = 5;

#[cfg(target_arch = "x86")]
const DESCRIPTORS: usize = 6;
#[cfg(target_arch = "x86_64")]
const DESCRIPTORS: usize = 7;

#[no_mangle]
static mut DESCRIPTOR_TABLE: [Descriptor; DESCRIPTORS] = [
    // NULL
    Descriptor::zeroed(),
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
    Descriptor::zeroed(),
    #[cfg(target_arch = "x86_64")]
    Descriptor::zeroed(),
];

#[repr(C, packed(2))]
struct DescriptorTableRegister {
    size: u16,
    offset: usize,
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
    const fn zeroed() -> Self {
        unsafe { mem::MaybeUninit::zeroed().assume_init() }
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

#[used]
static TASK_STATE_SEGMENT: TaskStateSegment = TaskStateSegment::zeroed();

#[cfg(target_arch = "x86")]
#[repr(C)]
struct TaskStateSegment {
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
struct TaskStateSegment {
    _reserved_0: u32,
    privilege_stack_table: [u64; 3],
    _reserved_1: u64,
    interrupt_stack_table: [u64; 7],
    _reserved_2: u64,
    _reserved_3: u16,
    iopb: u16,
}

impl TaskStateSegment {
    const fn zeroed() -> Self {
        unsafe { mem::MaybeUninit::zeroed().assume_init() }
    }
}

pub fn init() {
    // Correct the GDT address
    #[cfg(target_arch = "x86_64")]
    unsafe {
        let gdtr = DescriptorTableRegister {
            size: (mem::size_of_val(&DESCRIPTOR_TABLE) - 1) as u16,
            offset: (ptr::addr_of!(DESCRIPTOR_TABLE)) as usize,
        };
        arch::asm!(
            "lgdt [{}]", in(reg) &gdtr, options(readonly, nostack, preserves_flags)
        )
    }

    // Setup the TSS
    let addr = (ptr::addr_of!(TASK_STATE_SEGMENT)) as usize;
    let size = mem::size_of::<TaskStateSegment>() - 1;
    unsafe {
        DESCRIPTOR_TABLE[DESCRIPTOR_TSS as usize] = Descriptor::new(
            addr as u32,
            size as u32,
            DescriptorAccess::A
                .union(DescriptorAccess::E)
                .union(DescriptorAccess::P),
            0,
            DescriptorFlags::empty(),
        );
    }
    #[cfg(target_arch = "x86_64")]
    unsafe {
        DESCRIPTOR_TABLE[DESCRIPTOR_TSS as usize + 1].limit_0_15 = (addr >> 32) as u16;
        DESCRIPTOR_TABLE[DESCRIPTOR_TSS as usize + 1].base_0_15 = (addr >> 48) as u16;
    }

    // Update the TSS
    unsafe {
        arch::asm!("ltr {0:x}", in(reg) (DESCRIPTOR_TSS << 3) as u32, options(nostack, preserves_flags))
    }
}
