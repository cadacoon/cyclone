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

use core::mem;

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
        #[cfg(target_arch = "x86_64")] ist: u8,
        dpl: u8,
    ) -> Self {
        Self {
            offset_0_15: offset as u16,
            seg_sel,
            #[cfg(target_arch = "x86")]
            ist: 0,
            #[cfg(target_arch = "x86_64")]
            ist,
            gate_type_dpl_p: gate_type as u8 | dpl << 5,
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

const DE: usize = 0x00;
const DB: usize = 0x01;
const BP: usize = 0x03;
const OF: usize = 0x04;
const BR: usize = 0x05;
const UD: usize = 0x06;
const NM: usize = 0x07;
const DF: usize = 0x08;
const TS: usize = 0x0A;
const NP: usize = 0x0B;
const SS: usize = 0x0C;
const GP: usize = 0x0D;
const PF: usize = 0x0E;
const MF: usize = 0x10;
const AC: usize = 0x11;
const MC: usize = 0x12;
const XF: usize = 0x13;
const VE: usize = 0x14;
const CP: usize = 0x15;
const HV: usize = 0x1C;
const VC: usize = 0x1D;
const SX: usize = 0x1E;

pub(crate) fn init() {
    let mut descriptor_table = [Descriptor::zeroed(); 32];
}

extern "x86-interrupt" fn double_fault(ip: usize, cs: u16, flags: usize) {}
