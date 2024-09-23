use core::{marker, ops};

pub const GRANULARITY: usize = 0x1000;

#[cfg(target_arch = "x86")]
pub const PAGE_TABLE: *mut Table<Level2> = 0xFFFFF000 as *mut _;
#[cfg(target_arch = "x86_64")]
pub const PAGE_TABLE: *mut Table<Level4> = 0xFFFFFFFFFFFFF000 as *mut _;

#[repr(C, align(4096))]
pub struct Table<L: Level> {
    #[cfg(target_arch = "x86")]
    entries: [Entry; 1024],
    #[cfg(target_arch = "x86_64")]
    entries: [Entry; 512],
    level: marker::PhantomData<L>,
}

impl<L> Table<L>
where
    L: Level,
{
    fn init(&mut self) {
        for entry in &mut self.entries {
            entry.unmap();
        }
    }
}

impl<L> ops::Index<usize> for Table<L>
where
    L: Level,
{
    type Output = Entry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl<L> ops::IndexMut<usize> for Table<L>
where
    L: Level,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

impl<L> Table<L>
where
    L: HierarchicalLevel,
{
    pub fn table(&mut self, index: usize) -> Option<&mut Table<L::NextLevel>> {
        let entry = self.entries[index];
        if entry.free() {
            return None;
        }

        let addr = self as *mut _ as usize;
        #[cfg(target_arch = "x86")]
        let next_addr = addr << 10 | (index << 12);
        #[cfg(target_arch = "x86_64")]
        let next_addr = addr << 9 | (index << 12);
        Some(unsafe { &mut *(next_addr as *mut Table<L::NextLevel>) })
    }

    pub fn table_create(&mut self, index: usize) -> &mut Table<L::NextLevel> {
        if self.table(index).is_none() {
            let mut phys_mem = super::PHYS_MEM.lock();
            let frame = phys_mem.find_free(1).unwrap();
            phys_mem.mark_used(frame, 1);

            self.entries[index].map(frame);
            unsafe { self.table(index).unwrap_unchecked() }.init();
        }

        unsafe { self.table(index).unwrap_unchecked() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Entry(u32);

impl Entry {
    const FREE: u32 = 0;
    const PRESENT: u32 = 1 << 0;
    const WRITEABLE: u32 = 1 << 1;

    #[inline(always)]
    pub fn free(&self) -> bool {
        self.0 == Self::FREE
    }

    #[inline(always)]
    pub fn map(&mut self, frame: usize) {
        self.0 = (frame << 12) as u32 | Self::PRESENT | Self::WRITEABLE;
    }

    #[inline(always)]
    pub fn unmap(&mut self) -> usize {
        let frame = (self.0 >> 12) as usize;
        self.0 = Self::FREE;
        frame
    }
}

pub trait Level {}

pub enum Level1 {}
pub enum Level2 {}
#[cfg(target_arch = "x86_64")]
pub enum Level3 {}
#[cfg(target_arch = "x86_64")]
pub enum Level4 {}

impl Level for Level1 {}
impl Level for Level2 {}
#[cfg(target_arch = "x86_64")]
impl Level for Level3 {}
#[cfg(target_arch = "x86_64")]
impl Level for Level4 {}

pub trait HierarchicalLevel: Level {
    type NextLevel: Level;
}

impl HierarchicalLevel for Level2 {
    type NextLevel = Level1;
}

#[cfg(target_arch = "x86_64")]
impl HierarchicalLevel for Level3 {
    type NextLevel = Level2;
}

#[cfg(target_arch = "x86_64")]
impl HierarchicalLevel for Level4 {
    type NextLevel = Level3;
}
