use core::{marker, ops};

pub const GRANULARITY: usize = 4096;
pub const ROOT: *mut Table<Level2> = 0xFFFF_F000 as *mut _;

pub enum Level2 {}
pub enum Level1 {}

pub trait Level {}

impl Level for Level2 {}
impl Level for Level1 {}

pub trait HierarchicalLevel: Level {
    type NextLevel: Level;
}

impl HierarchicalLevel for Level2 {
    type NextLevel = Level1;
}

#[repr(C, align(4096))]
pub struct Table<L: Level> {
    entries: [Entry; 1024],
    level: marker::PhantomData<L>,
}

impl<L> Table<L>
where
    L: Level,
{
    pub const fn new() -> Self {
        Self {
            entries: [Entry(0); 1024],
            level: marker::PhantomData,
        }
    }

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
        let next_addr = addr << 10 | (index << 12);
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
