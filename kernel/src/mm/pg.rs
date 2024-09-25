use core::{marker, ops};

pub const BYTES_PER_PAGE: usize = 4096;

#[cfg(target_arch = "x86")]
pub const PAGES_PER_TABLE: usize = 1024;
#[cfg(target_arch = "x86")]
pub const PAGES_TOTAL: usize = 0xFFFFF;

#[cfg(target_arch = "x86_64")]
pub const PAGES_PER_TABLE: usize = 512;
#[cfg(target_arch = "x86_64")]
pub const PAGES_TOTAL: usize = 0xFFFFFFFFF;

#[cfg(target_arch = "x86")]
pub const PAGE_TABLE: *mut PageTable<Level2> = 0xFFFFF000 as *mut _;
#[cfg(target_arch = "x86_64")]
pub const PAGE_TABLE: *mut PageTable<Level4> = 0o177_777_776_776_776_776_0000 as *mut _;

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Page(pub usize);

#[repr(C, align(4096))]
pub struct PageTable<L: Level> {
    entries: [PageTableEntry; PAGES_PER_TABLE],
    level: marker::PhantomData<L>,
}

impl<L> PageTable<L>
where
    L: Level,
{
    fn init(&mut self) {
        for entry in &mut self.entries {
            entry.unmap();
        }
    }
}

impl<L> ops::Index<Page> for PageTable<L>
where
    L: Level,
{
    type Output = PageTableEntry;

    fn index(&self, index: Page) -> &Self::Output {
        &self.entries[L::index(index)]
    }
}

impl<L> ops::IndexMut<Page> for PageTable<L>
where
    L: Level,
{
    fn index_mut(&mut self, index: Page) -> &mut Self::Output {
        &mut self.entries[L::index(index)]
    }
}

impl<L> PageTable<L>
where
    L: HierarchicalLevel,
{
    pub fn table(&mut self, page: Page) -> Option<&mut PageTable<L::NextLevel>> {
        let entry = self.entries[L::index(page)];
        if entry.free() {
            return None;
        }

        let addr = self as *mut _ as usize;
        #[cfg(target_arch = "x86")]
        let next_addr = addr << 10 | L::index(page) << 12;
        #[cfg(target_arch = "x86_64")]
        let next_addr = { (((addr << 9 | L::index(page) << 12) << 16) as i64 >> 16) as usize };
        Some(unsafe { &mut *(next_addr as *mut PageTable<L::NextLevel>) })
    }

    pub fn table_create(&mut self, page: Page) -> &mut PageTable<L::NextLevel> {
        if self.table(page).is_none() {
            let mut phys_mem = super::PHYS_MEM.lock();
            let frame = phys_mem.find_free(1).unwrap();
            phys_mem.mark_used(frame, 1);

            self.entries[L::index(page)].map(frame);
            unsafe { self.table(page).unwrap_unchecked() }.init();
        }

        unsafe { self.table(page).unwrap_unchecked() }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PageTableEntry(usize);

impl PageTableEntry {
    const FREE: usize = 0;
    const PRESENT: usize = 1 << 0;
    const WRITEABLE: usize = 1 << 1;

    #[inline(always)]
    pub fn free(&self) -> bool {
        self.0 == Self::FREE
    }

    #[inline(always)]
    pub fn map(&mut self, frame: usize) {
        self.0 = Self::PRESENT | Self::WRITEABLE | frame << 12;
    }

    #[inline(always)]
    pub fn unmap(&mut self) -> usize {
        let frame = (self.0 >> 12) as usize;
        self.0 = Self::FREE;
        frame
    }
}

pub trait Level {
    fn index(page: Page) -> usize;
}

pub enum Level1 {}
pub enum Level2 {}
#[cfg(target_arch = "x86_64")]
pub enum Level3 {}
#[cfg(target_arch = "x86_64")]
pub enum Level4 {}

impl Level for Level1 {
    fn index(page: Page) -> usize {
        if cfg!(target_arch = "x86") {
            page.0 >> 10 * 0 & (1 << 10) - 1
        } else {
            page.0 >> 9 * 0 & (1 << 9) - 1
        }
    }
}
impl Level for Level2 {
    fn index(page: Page) -> usize {
        if cfg!(target_arch = "x86") {
            page.0 >> 10 * 1 & (1 << 10) - 1
        } else {
            page.0 >> 9 * 1 & (1 << 9) - 1
        }
    }
}
#[cfg(target_arch = "x86_64")]
impl Level for Level3 {
    fn index(page: Page) -> usize {
        page.0 >> 9 * 2 & (1 << 9) - 1
    }
}
#[cfg(target_arch = "x86_64")]
impl Level for Level4 {
    fn index(page: Page) -> usize {
        page.0 >> 9 * 3 & (1 << 9) - 1
    }
}

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
