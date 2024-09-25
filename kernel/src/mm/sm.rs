#[no_mangle]
static DESCRIPTOR_TABLE: [Descriptor; 6] = [
    // Null
    Descriptor::new(
        0x00000000,
        0x00000,
        DescriptorAccess::empty(),
        0,
        DescriptorFlags::empty(),
    ),
    // Kernel code
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
    // Kernel data
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
    // User code
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
    // User data
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
    Descriptor::empty(),
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
    const fn empty() -> Self {
        Self {
            limit_0_15: 0,
            base_0_15: 0,
            base_16_23: 0,
            access: 0,
            flags_and_limit_16_19: 0,
            base_24_31: 0,
        }
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
    #[derive(Copy, Clone, Debug)]
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
