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

use core::{arch, mem, ptr};

use crate::mm;

static mut DESCRIPTOR_TABLE: [Descriptor; 32] = [Descriptor::zeroed(); 32];

#[repr(C, packed(2))]
struct DescriptorTableRegister {
    size: u16,
    offset: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Descriptor {
    offset_0_15: u16,
    seg_sel: u16,
    ist: u8,
    gate_type_dpl_p: u8,
    offset_16_31: u16,
    #[cfg(target_arch = "x86_64")]
    offset_32_63: u32,
    #[cfg(target_arch = "x86_64")]
    _reserved: u32,
}

impl Descriptor {
    const fn zeroed() -> Self {
        unsafe { mem::MaybeUninit::zeroed().assume_init() }
    }

    const fn new(
        offset: usize,
        seg_sel: u16,
        gate_type: DescriptorGateType,
        ist: u8,
        dpl: u8,
    ) -> Self {
        Self {
            offset_0_15: offset as u16,
            seg_sel,
            ist,
            gate_type_dpl_p: gate_type as u8 | dpl << 5 | 1 << 7,
            offset_16_31: (offset >> 16) as u16,
            #[cfg(target_arch = "x86_64")]
            offset_32_63: (offset >> 32) as u32,
            #[cfg(target_arch = "x86_64")]
            _reserved: 0,
        }
    }
}

#[repr(u8)]
enum DescriptorGateType {
    Interrupt = 0xE,
    Trap = 0xF,
}

const DE: usize = 0x00; // Division Error
const DB: usize = 0x01; // Debug
const BP: usize = 0x03; // Breakpoint
const OF: usize = 0x04; // Overflow
const BR: usize = 0x05; // Bound Range Exceeded
const UD: usize = 0x06; // Invalid Opcode
const NM: usize = 0x07; // Device Not Available
const DF: usize = 0x08; // Double Fault
const TS: usize = 0x0A; // Invalid TSS
const NP: usize = 0x0B; // Segment Not Present
const SS: usize = 0x0C; // Stack-Segment Fault
const GP: usize = 0x0D; // General Protection Fault
const PF: usize = 0x0E; // Page Fault
const MF: usize = 0x10; // x87 Floating-Point Exception
const AC: usize = 0x11; // Alignment Check
const MC: usize = 0x12; // Machine Check
const XF: usize = 0x13; // SIMD Floating-Point Exception
const VE: usize = 0x14; // Virtualization Exception
const CP: usize = 0x15; // Control Protection Exception
const HV: usize = 0x1C; // Hypervisor Injection Exception
const VC: usize = 0x1D; // VMM Communication Exception
const SX: usize = 0x1E; // Security Exception

pub(crate) fn init() {
    // Setup the IDT
    unsafe {
        DESCRIPTOR_TABLE[DB] = Descriptor::new(
            double_fault as usize,
            mm::sm::DESCRIPTOR_KCODE << 3,
            DescriptorGateType::Interrupt,
            0,
            0,
        )
    };

    // Update the IDT
    unsafe {
        let idtr = DescriptorTableRegister {
            size: (mem::size_of_val(&DESCRIPTOR_TABLE) - 1) as u16,
            offset: (ptr::addr_of!(DESCRIPTOR_TABLE)) as usize,
        };
        arch::asm!(
            "lidt [{}]", in(reg) &idtr, options(readonly, nostack, preserves_flags)
        )
    }
}

#[repr(C)]
#[derive(Debug)]
struct StackFrame {
    ip: usize,
    cs: u16,
    flags: usize,
    #[cfg(target_arch = "x86_64")]
    sp: usize,
    #[cfg(target_arch = "x86_64")]
    ss: u16,
}

extern "x86-interrupt" fn double_fault(_stack_frame: StackFrame) {}
