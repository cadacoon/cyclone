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

use core::{hint, marker, mem::MaybeUninit, panic};

use bitflags::bitflags;
use pio::Port;

fn main() {}

#[panic_handler]
fn panic(_info: &panic::PanicInfo) -> ! {
    loop {
        hint::spin_loop();
    }
}

trait ConfigurationAccessMechanism: Sized {
    fn header(&self, location: u16) -> ConfigurationSpaceHeader<Self>;
}

struct CAM(Port<u32>, Port<u32>);

impl ConfigurationAccessMechanism for CAM {
    fn header(&self, location: u16) -> ConfigurationSpaceHeader<Self> {
        let mut header: MaybeUninit<ConfigurationSpaceHeader<Self>> = MaybeUninit::uninit();
        for register in 0..size_of::<ConfigurationSpaceHeader<Self>>() / size_of::<u32>() {
            self.0
                .write(1 << 31 | (location as u32) << 8 | (register * size_of::<u32>()) as u32);
            let value = self.1.read();
            unsafe {
                (header.as_mut_ptr() as *mut u32).add(register).write(value);
            }
        }
        unsafe { header.assume_init() }
    }
}

struct ECAM(*mut u32);

impl ConfigurationAccessMechanism for ECAM {
    fn header(&self, location: u16) -> ConfigurationSpaceHeader<Self> {
        let mut header: MaybeUninit<ConfigurationSpaceHeader<Self>> = MaybeUninit::uninit();
        for register in 0..size_of::<ConfigurationSpaceHeader<Self>>() {
            let value = unsafe {
                self.0
                    .add((location as usize) << 12 | (register * size_of::<u32>()))
                    .read_volatile()
            };
            unsafe {
                (header.as_mut_ptr() as *mut u32).add(register).write(value);
            }
        }
        unsafe { header.assume_init() }
    }
}

#[repr(C)]
struct ConfigurationSpaceHeader<AM: ConfigurationAccessMechanism> {
    vendor_id: u16,
    device_id: u16,
    command: ConfigurationSpaceHeaderCommand,
    status: ConfigurationSpaceHeaderStatus,

    revision_id: u8,
    class_code: [u8; 3],

    cache_line_size: u8,
    latency_timer: u8,
    /// 0-6 Header Layout
    ///   7 Multi-Function Device
    header_type: u8,
    /// 0-3 Completion Code
    ///   6 Start BIST
    ///   7 BIST Capable
    bist: u8,

    type_specific: ConfigurationSpaceHeaderTypeSpecific,

    _access_mechanism: marker::PhantomData<AM>,
}

bitflags! {
    struct ConfigurationSpaceHeaderCommand: u16 {
        /// I/O Space Enable
        const IOSE = 1 << 0;
        /// Memory Space Enable
        const MSE = 1 << 1;
        /// Bus Master Enable
        const BME = 1 << 2;
        /// Special Cycle Enable
        const SCE = 1 << 3;
        /// Memory Write and Invalidate
        const MWI = 1 << 4;
        /// VGA Palette Snoop
        const VGAPS = 1 << 5;
        /// Parity Error Response
        const PER = 1 << 6;
        /// IDSEL Stepping/Wait Cycle Control
        const IDSEL = 1 << 7;
        /// SERR# Enable
        const SERRE = 1 << 8;
        /// Fast Back-to-Back Transactions Enable
        const FB2BTE = 1 << 9;
        /// Interrupt Disable
        const ID = 1 << 10;
    }

    #[derive(Clone, Copy)]
    struct ConfigurationSpaceHeaderStatus: u16 {
        /// Immediate Readiness
        const IR = 1 << 0;
        /// Interrupt Status
        const IS = 1 << 3;
        /// Capabilities List
        const CL = 1 << 4;
        /// 66 MHz Capable
        const _66C = 1 << 5;
        /// Fast Back-to-Back Transactions Capable
        const FB2BTC = 1 << 7;
        /// Master Data Parity Error
        const MDPE = 1 << 8;
        /// DEVSEL Timing
        const DEVSEL = ((1 << 2) - 1) << 9;
        /// Signaled Target Abort
        const STA = 1 << 11;
        /// Received Target Abort
        const RTA = 1 << 12;
        /// Received Master Abort
        const RMA = 1 << 13;
        /// Received/Signaled System Error
        const SE = 1 << 14;
        /// Detected Parity Error
        const DPE = 1 << 15;
    }
}

union ConfigurationSpaceHeaderTypeSpecific {
    type_0: ConfigurationSpaceHeaderType0Specific,
    type_1: ConfigurationSpaceHeaderType1Specific,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ConfigurationSpaceHeaderType0Specific {
    /// 0 Indicator
    /// 1 Memory Type (16-bit)
    /// 2 Memory Type (64-bit)
    /// 3 Prefetchable
    base_address_register: [u32; 6],
    cardbus_cis_pointer: u32,

    subsystem_vendor_id: u16,
    subsystem_id: u16,

    ///    00 Expansion ROM Enable
    /// 01-03 Expansion ROM Validation Status
    /// 04-07 Expansion ROM Validation Details
    /// 11-31 Expansion ROM Base Address
    expansion_rom_base_address: u32,

    capabilities_pointer: u8,
    _reserved: [u8; 7],

    interrupt_line: u8,
    interrupt_pin: u8,
    min_gnt: u8,
    max_lat: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ConfigurationSpaceHeaderType1Specific {
    /// 0 Indicator
    /// 1 Memory Type (16-bit)
    /// 2 Memory Type (64-bit)
    /// 3 Prefetchable
    base_address_register: [u32; 2],

    primary_bus_number: u8,
    secondary_bus_number: u8,
    subordinate_bus_number: u8,
    secondary_latency_timer: u8,

    io_base: u8,
    io_limit: u8,
    secondary_status: ConfigurationSpaceHeaderStatus,

    memory_base: u16,
    memory_limit: u16,
    prefetchable_memory_base: u16,
    prefetchable_memory_limit: u16,

    prefetchable_memory_base_upper: u32,
    prefetchable_memory_limit_upper: u32,

    io_base_upper: u16,
    io_limit_upper: u16,

    capabilites_pointer: u8,
    _reserved: [u8; 3],

    ///    00 Expansion ROM Enable
    /// 01-03 Expansion ROM Validation Status
    /// 04-07 Expansion ROM Validation Details
    /// 11-31 Expansion ROM Base Address
    expansion_rom_base_address: u32,

    interrupt_line: u8,
    interrupt_pin: u8,
    bridge_control: ConfigurationSpaceHeaderBridgeControl,
}

bitflags! {
    #[derive(Clone, Copy)]
    struct ConfigurationSpaceHeaderBridgeControl: u16 {
        /// Parity Error Response Enable
        const PERE = 1 << 0;
        /// SERR# Enable
        const SERRE = 1 << 1;
        /// ISA Enable
        const ISAE = 1 << 2;
        /// VGA Enable
        const VGAE = 1 << 3;
        /// VGA 16-bit Decode
        const VGA16D = 1 << 4;
        /// Master Abort Mode
        const MAM = 1 << 5;
        /// Secondary Bus Reset
        const SBR = 1 << 6;
        /// Fast Back-to-Back Transactions Enable
        const FB2BTE = 1 << 7;
        /// Primary Discard Timer
        const PDT = 1 << 8;
        /// Secondary Discard Timer
        const SDT = 1 << 9;
        /// Discard Timer Status
        const DTS = 1 << 10;
        /// Discard Timer SERR# Enable
        const DTSERRE = 1 << 11;
    }
}
