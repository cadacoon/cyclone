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

use crate::ex;

use super::Port;

static mut DESCRIPTOR_TABLE: [Descriptor; 32 + 16] = [Descriptor::zeroed(); 32 + 16];

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

pub static PIC: PIC = unsafe {
    PIC {
        cmd1: Port::new(0x20),
        dat1: Port::new(0x21),
        cmd2: Port::new(0xA0),
        dat2: Port::new(0xA1),
    }
};

pub struct PIC {
    cmd1: Port<u8>,
    dat1: Port<u8>,
    cmd2: Port<u8>,
    dat2: Port<u8>,
}

impl PIC {
    const ICW1_ICW4: u8 = 1 << 0;
    const ICW1: u8 = 1 << 4;
    const ICW4_8086: u8 = 1 << 0;
    const OCW2_EOI: u8 = 1 << 5;

    pub fn init(&self) {
        // ICW1
        self.cmd1.write(Self::ICW1 | Self::ICW1_ICW4);
        self.cmd2.write(Self::ICW1 | Self::ICW1_ICW4);

        // ICW2
        self.dat1.write(0x20);
        self.dat2.write(0x20 + 8);

        // ICW3
        self.dat1.write(1 << 2); // Line
        self.dat2.write(1 << 1); // Mask

        // ICW4
        self.dat1.write(Self::ICW4_8086);
        self.dat2.write(Self::ICW4_8086);
    }

    pub fn eoi(&self, irq: u8) {
        // OCW2
        if irq >= 8 {
            self.cmd2.write(Self::OCW2_EOI);
        }
        self.cmd1.write(Self::OCW2_EOI);
    }
}

pub fn init() {
    init_ivt();
    PIC.init();

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
    sp: usize,
    ss: u16,
}

macro_rules! ivt {
    ($($vector:tt $name:ident $description:tt $function:stmt),*$(,)?) => {
        fn init_ivt() {
            unsafe {
                $(DESCRIPTOR_TABLE[$vector] = Descriptor::new($name as usize, 1 << 3, DescriptorGateType::Interrupt, 0, 0);)*
            }
        }

        $(extern "x86-interrupt" fn $name(_stack_frame: StackFrame) {
            $function
        })*
    };
}

ivt!(
    0x00 exc_de "Division Error" {},
    0x01 exc_db "Debug" {},
    0x02 exc_02 "Exception 2" {},
    0x03 exc_bp "Breakpoint" {},
    0x04 exc_of "Overflow" {},
    0x05 exc_br "Bound Range Exceeded" {},
    0x06 exc_ud "Invalid Opcode" {},
    0x07 exc_nm "Device Not Available" {},
    0x08 exc_df "Double Fault" {},
    0x09 exc_09 "Exception 9" {},
    0x0A exc_ts "Invalid TSS" {},
    0x0B exc_np "Segment Not Present" {},
    0x0C exc_ss "Stack-Segment Fault" {},
    0x0D exc_gp "General Protection Fault" {},
    0x0E exc_pf "Page Fault" {},
    0x0F exc_15 "Exception 15" {},
    0x10 exc_mf "x87 Floating-Point Exception" {},
    0x11 exc_ac "Alignment Check" {},
    0x12 exc_mc "Machine Check" {},
    0x13 exc_xf "SIMD Floating-Point Exception" {},
    0x14 exc_ve "Virtualization Exception" {},
    0x15 exc_cp "Control Protection Exception" {},
    0x16 exc_22 "Exception 22" {},
    0x17 exc_23 "Exception 23" {},
    0x18 exc_24 "Exception 24" {},
    0x19 exc_25 "Exception 25" {},
    0x1A exc_26 "Exception 26" {},
    0x1B exc_27 "Exception 27" {},
    0x1C exc_hv "Hypervisor Injection Exception" {},
    0x1D exc_vc "VMM Communication Exception" {},
    0x1E exc_sx "Security Exception" {},
    0x1F exc_31 "Exception 31" {},
    0x20 irq_00 "IRQ 0" {
        PIC.eoi(0);

        unsafe { ex::Scheduler::get() }.enter(true);
    },
    0x21 irq_01 "IRQ 1" {
        PIC.eoi(1);
    },
    0x22 irq_02 "IRQ 2" {
        PIC.eoi(2);
    },
    0x23 irq_03 "IRQ 3" {
        PIC.eoi(3);
    },
    0x24 irq_04 "IRQ 4" {
        PIC.eoi(4);
    },
    0x25 irq_05 "IRQ 5" {
        PIC.eoi(5);
    },
    0x26 irq_06 "IRQ 6" {
        PIC.eoi(6);
    },
    0x27 irq_07 "IRQ 7" {
        PIC.eoi(7);
    },
    0x28 irq_08 "IRQ 8" {
        PIC.eoi(8);
    },
    0x29 irq_09 "IRQ 9" {
        PIC.eoi(9);
    },
    0x2A irq_10 "IRQ 10" {
        PIC.eoi(10);
    },
    0x2B irq_11 "IRQ 11" {
        PIC.eoi(11);
    },
    0x2C irq_12 "IRQ 12" {
        PIC.eoi(12);
    },
    0x2D irq_13 "IRQ 13" {
        PIC.eoi(13);
    },
    0x2E irq_14 "IRQ 14" {
        PIC.eoi(14);
    },
    0x2F irq_15 "IRQ 15" {
        PIC.eoi(15);
    },
);
