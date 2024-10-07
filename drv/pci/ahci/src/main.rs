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

use core::{hint, panic};

use bitflags::bitflags;
use drv_pci::Device;

fn main(_device: Device) {}

#[panic_handler]
fn panic(_info: &panic::PanicInfo) -> ! {
    loop {
        hint::spin_loop();
    }
}

#[repr(C)]
struct HBA {
    /// Host Capabilities
    cap: HBACapabilities,
    /// Global Host Control
    ghc: HBAGlobalControl,
    /// Interrupt Status
    is: u32,
    /// Ports Implemented
    pi: u32,
    /// Version: Minor Version Number
    vs_mnr: u16,
    /// Version: Major Version Number
    vs_mjr: u16,
    /// Command Completion Coalescing Control: Enable, Interrupt
    ccc_ctl_enint: u8,
    /// Command Completion Coalescing Control: Command Completions
    ccc_ctl_cc: u8,
    /// Command Completion Coalescing Control: Timeout Value
    ccc_ctl_tv: u16,
    /// Command Completion Coalescing Ports
    ccc_ports: u32,
    /// Enclosure Management Location: Buffer Size
    em_loc_sz: u32,
    /// Enclosure Management Location: Offset
    em_loc_ofst: u32,
    /// Enclosure Management Control
    em_ctl: HBAEnclosureManagementControl,
    /// Host Capabilities Extended
    cap2: HBACapabilitiesExtended,
    /// BIOS/OS Handoff Control and Status
    bohc: HBABIOSOSHandoffControl,
    /// Reserved
    _rsvd: [u8; 52],
    /// Reserved for NVMHCI
    _rsvd_nvmhci: [u8; 64],
    /// Vendor Specific
    _rsvd_vendor: [u8; 96],
    port: [HBAPort; 32],
}

bitflags! {
    struct HBACapabilities: u32 {
        /// Number of Ports
        const NP = (1 << 4) - 1;
        /// Supports External SATA
        const SXS = 1 << 5;
        /// Enclosure Management Supported
        const EMS = 1 << 6;
        /// Command Completion Coalescing Supported
        const CCCS = 1 << 7;
        /// Number of Command Slots
        const NCS = ((1 << 4) - 1) << 8;
        /// Partial State Capable
        const PSC = 1 << 13;
        /// Slumber State Capable
        const SSC = 1 << 14;
        /// PIO Multiple DRQ Block
        const PMD = 1 << 15;
        /// FIS-based Switching Supported
        const FBSS = 1 << 16;
        /// Supports Port Multiplier
        const SPM = 1 << 17;
        /// Supports AHCI mode only
        const SAM = 1 << 18;
        /// Interface Speed Support Gen 1 (1.5 Gbps)
        const ISS_1 = 1 << 20;
        /// Interface Speed Support Gen 2 (3 Gbps)
        const ISS_2 = 2 << 20;
        /// Interface Speed Support Gen 3 (6 Gbps)
        const ISS_3 = 3 << 20;
        /// Supports Command List Override
        const SCLO = 1 << 24;
        /// Supports Activity LED
        const SAL = 1 << 25;
        /// Supports Aggressive Link Power Management
        const SALP = 1 << 26;
        /// Supports Staggered Spin-up
        const SSS = 1 << 27;
        /// Supports Mechanical Presence Switch
        const SMPS = 1 << 28;
        /// Supports SNotification Register
        const SSNTF = 1 << 29;
        /// Supports Native Command Queuing
        const SNCQ = 1 << 30;
        /// Supports 64-bit Addressing
        const S64A = 1 << 31;
    }

    struct HBACapabilitiesExtended: u32 {
        /// BIOS/OS Handoff
        const BOH = 1 << 0;
        /// NVMHCI Present
        const NVMP = 1 << 1;
        /// Automatic Partial to Slumber Transitions
        const APST = 1 << 2;
        /// Supports Device Sleep
        const SDS = 1 << 3;
        /// Supports Aggressive Device Sleep Management
        const SADM = 1 << 4;
        /// DevSleep Entrance from Slumber Only
        const DESO = 1 << 5;
    }

    struct HBAGlobalControl: u32 {
        /// HBA Reset
        const HR = 1 << 0;
        /// Interrupt Enable
        const IE = 1 << 1;
        /// MSI Revert to Single Message
        const MRSM = 1 << 2;
        /// AHCI Enable
        const AE = 1 << 31;
    }

    struct HBAEnclosureManagementControl: u32 {
        /// Message Received
        const STS_RM = 1 << 0;
        /// Transmit Message
        const CTL_TM = 1 << 8;
        /// Reset
        const CTL_RST = 1 << 9;
        /// LED Message Types
        const SUPP_LED = 1 << 16;
        /// SAF-TE Enclosure Management Messages
        const SUPP_SAFTE = 1 << 17;
        /// SES-2 Enclosure Management Messages
        const SUPP_SES2 = 1 << 18;
        /// SGPIO Enclosure Management Messages
        const SUPP_SGPIO = 1 << 19;
        /// Single Message Buffer
        const ATTR_SMB = 1 << 24;
        /// Transmit Only
        const ATTR_XMT = 1 << 25;
        /// Activity LED Hardware Driven
        const ATTR_ALHD = 1 << 26;
        /// Port Multiplier Support
        const ATTR_PM = 1 << 27;
    }

    struct HBABIOSOSHandoffControl: u32 {
        /// BIOS Owned Semaphore
        const BOS = 1 << 0;
        /// OS Owned Semaphore
        const OOS = 1 << 1;
        /// SMI on OS Ownership Change Enable
        const SOOE = 1 << 2;
        /// OS Ownership Change
        const OOC = 1 << 3;
        /// BIOS Busy
        const BB = 1 << 4;
    }
}

#[repr(C)]
struct HBAPort {
    /// Command List Base Address
    clb: u32,
    /// Command List Base Address Upper 32-bits
    clbu: u32,
    /// FIS Base Address
    fb: u32,
    /// FIS Base Address Upper 32-bits
    fbu: u32,
    /// Interrupt Status
    is: HBAPortInterrupt,
    /// Interrupt Enable
    ie: HBAPortInterrupt,
    /// Command and Status
    cmd: HBAPortCommand,
    /// Reserved
    _rsvd_0: u32,
    /// Task File Data: Status
    tfd_sts: u8,
    /// Task File Data: Error
    tfd_err: u8,
    /// Task File Data: Reserved
    _tfd_rsvd: u16,
    /// Signature
    sig: u32,
    /// Serial ATA Status
    ssts: u32,
    /// Serial ATA Control
    sctl: u32,
    /// Serial ATA Error
    serr: u32,
    /// Serial ATA Active
    sact: u32,
    /// Command Issue
    ci: u32,
    /// Serial ATA Notification
    sntf: u32,
    /// FIS-based Switching Control
    fbs: u32,
    /// Device Sleep
    devslp: u32,
    /// Reserved
    _rsvd_1: [u8; 40],
    /// Vendor Specific
    _rsvd_vendor: [u8; 16],
}

bitflags! {
    struct HBAPortInterrupt: u32 {
        /// Device to Host Register FIS Interrupt
        const DHR = 1 << 0;
        /// PIO Setup FIS Interrupt
        const PS = 1 << 1;
        /// DMA Setup FIS Interrupt
        const DS = 1 << 2;
        /// Set Device Bits Interrupt
        const SDB = 1 << 3;
        /// Unknown FIS Interrupt
        const UF = 1 << 4;
        /// Descriptor Processed
        const DP = 1 << 5;
        /// Port Connect Change
        const PC = 1 << 6;
        /// Device Mechanical Presence
        const DMP = 1 << 7;
        /// PhyRdy Change
        const PRC = 1 << 22;
        /// Incorrect Port Multiplier
        const IPM = 1 << 23;
        /// Overflow
        const OF = 1 << 24;
        /// Interface Non-fatal Error
        const INF = 1 << 26;
        /// Interface Fatal Error
        const IF = 1 << 27;
        /// Host Bus Data Error
        const HBD = 1 << 28;
        /// Host Bus Fatal Error
        const HBF = 1 << 29;
        /// Task File Error
        const TFE = 1 << 30;
        /// Cold Port Detect
        const CPD = 1 << 31;
    }

    struct HBAPortCommand: u32 {
        /// Start
        const ST = 1 << 0;
        /// Spin-Up Device
        const SUD = 1 << 1;
        /// Power On Device
        const POD = 1 << 2;
        /// Command List Override
        const CLO = 1 << 3;
        /// FIS Receive Enable
        const FRE = 1 << 4;
        /// Current Command Slot
        const CCS = ((1 << 5) - 1) << 8;
        /// Mechanical Presence Switch State
        const MPSS = 1 << 13;
        /// FIS Receive Running
        const FR = 1 << 14;
        /// Command List Running
        const CR = 1 << 15;
        /// Cold Presence State
        const CPS = 1 << 16;
        /// Port Multiplier Attached
        const PMA = 1 << 17;
        /// Hot Plug Capable Port
        const HPCP = 1 << 18;
        /// Mechanical Presence Switch Attached to Port
        const MPSP = 1 << 19;
        /// Cold Presence Detection
        const CPD = 1 << 20;
        /// External SATA Port
        const ESP = 1 << 21;
        /// FIS-based Switching Capable Port
        const FBSCP = 1 << 22;
        /// Automatic Partial to Slumber Transitions Enabled
        const APSTE = 1 << 23;
        /// Device is ATAPI
        const ATAPI = 1 << 24;
        /// Drive LED on ATAPI Enable
        const DLAE = 1 << 25;
        /// Aggresive Link Power Management Enable
        const ALPE = 1 << 26;
        /// Aggressive Slumber / Partial
        const ASP = 1 << 27;
        /// Interface Communication Control
        const ICC = ((1 << 4) - 1) << 28;
    }
}

#[repr(C, align(1024))]
struct CommandList([CommandHeader; 32]);

#[repr(C)]
struct CommandHeader {
    /// 0-4 Command FIS Length
    ///   5 ATAPI
    ///   6 Write
    ///   7 Prefetchable
    cflawp: u8,
    ///   0 Reset
    ///   1 BIST
    ///   2 Clear Busy opon R_OK
    ///   3 Reserved
    /// 4-7 Port Multiplier Port
    rbcpmp: u8,
    /// Physical Region Descriptor Table Length
    prdtl: u16,
    /// Physical Region Descriptor Byte Count
    prdbc: u32,
    /// Command Table Base Address
    ctba: u32,
    /// Command Table Base Address Upper 32-bits
    ctbau: u32,
    /// Reserved
    _rsvd: [u32; 4],
}

#[repr(C, align(128))]
struct CommandTable {
    /// Command FIS
    cfis: H2DRegisterFIS,
    _cfis_remaining: [u32; 11],
    /// ATAPI Command
    acmd: [u32; 4],
    _reserved: [u32; 12],
    /// Physical Region Descriptor Table
    prdt: [PhysicalRegionDescriptor; 0],
}

#[repr(C)]
struct PhysicalRegionDescriptor {
    /// Data Base Address
    dba: u32,
    /// Data Base Address
    dbau: u32,
    /// Reserved
    _rsvd: u32,
    /// 00-21 Data Byte Count
    /// 22-30 Reserved
    ///    31 Interrupt on Completion
    dbci: u32,
}

#[repr(C, align(256))]
struct ReceivedFIS {
    /// DMA Setup FIS
    dsfis: DMASetupFIS,
    _reserved_0: u32,
    /// PIO Setup FIS
    psfis: PIOSetupFIS,
    _reserved_1: [u32; 3],
    /// D2H Register FIS
    rfis: D2HRegisterFIS,
    _reserved_2: u32,
    /// Set Device Bits FIS
    sdbfis: SetDeviceBitsFIS,
    /// Unknown FIS
    ufis: [u32; 16],
    _reserved_3: [u32; 24],
}

#[repr(C)]
struct H2DRegisterFIS {
    fis_type: u8,
    flags: u8,
    command: u8,
    features_0_7: u8,

    lba_0_7: u8,
    lba_8_15: u8,
    lba_16_32: u8,
    device: u8,

    lba_24_31: u8,
    lba_32_39: u8,
    lba_40_47: u8,
    features_8_15: u8,

    count_0_7: u8,
    count_8_15: u8,
    icc: u8,
    control: u8,

    auxiliary_0_7: u8,
    auxiliary_8_15: u8,
    _reserved: [u8; 2],
}

#[repr(C)]
struct DMASetupFIS {
    fis_type: u8,
    flags: u8,
    _reserved_0: [u8; 2],

    dma_buffer_identifier_low: u32,
    dma_buffer_identifier_high: u32,
    _reserved_1: u32,
    dma_buffer_offset: u32,
    dma_transfer_count: u32,
    _reserved_2: u32,
}

#[repr(C)]
struct PIOSetupFIS {
    fis_type: u8,
    flags: u8,
    status: u8,
    error: u8,

    lba_0_7: u8,
    lba_8_15: u8,
    lba_16_32: u8,
    device: u8,

    lba_24_31: u8,
    lba_32_39: u8,
    lba_40_47: u8,
    _reserved_0: u8,

    count_0_7: u8,
    count_8_15: u8,
    _reserved_1: u8,
    e_status: u8,

    transfer_count: u16,
    _reserved_2: [u8; 2],
}

#[repr(C)]
struct D2HRegisterFIS {
    fis_type: u8,
    flags: u8,
    status: u8,
    error: u8,

    lba_0_7: u8,
    lba_8_15: u8,
    lba_16_32: u8,
    device: u8,

    lba_24_31: u8,
    lba_32_39: u8,
    lba_40_47: u8,
    _reserved_0: u8,

    count_0_7: u8,
    count_8_15: u8,
    _reserved_1: [u8; 6],
}

#[repr(C)]
struct SetDeviceBitsFIS {
    fis_type: u8,
    flags: u8,
    status: u8,
    error: u8,

    _unknown: u32,
}
