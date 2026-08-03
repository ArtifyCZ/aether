use core::iter::Peekable;

/// Huge page size in bytes
///
/// A huge page is 1 GiB
pub const HUGE_PAGE_SIZE: usize = 512 * 512 * 4096;

/// Large page size in bytes
///
/// A large page is 2 MiB
pub const LARGE_PAGE_SIZE: usize = 512 * 4096;

#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub base_address: usize,
    pub length: usize,
    pub kind: MemoryRegionKind,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryRegionKind {
    /// Usable with no strings or requirements attached
    Usable,
    /// Can be used—reclaimed, but anything to be kept should be filtered-out first
    Reclaimable,
    /// Parts can theoretically be reclaimed, contains the kernel and modules (e.g., initrd)
    ExecutableAndModules,
    /// Should not be used
    Reserved,
    /// Should not be used, the memory is not reliable
    BadMemory,
}

#[derive(Debug)]
pub struct MemoryPage<const SIZE: usize> {
    pub base_address: usize,
}

fn align_up(n: usize, align: usize) -> usize {
    (n + align - 1) & !(align - 1)
}

fn is_aligned(n: usize, align: usize) -> bool {
    n & (align - 1) == 0
}

#[derive(Debug)]
pub enum MemoryRegionPage {
    Huge(MemoryPage<HUGE_PAGE_SIZE>),
    Large(MemoryPage<LARGE_PAGE_SIZE>),
}

#[derive(Debug, Clone)]
pub struct PagedMemoryRegionIterator<TIter: Iterator<Item = MemoryRegion>> {
    iter: Peekable<TIter>,
    current_region_end: usize,
    current_page_start: usize,
}

pub trait IntoPagedMemoryRegionIterator: Iterator<Item = MemoryRegion> + Sized {
    fn into_paged_iter(self) -> PagedMemoryRegionIterator<Self>;
}

impl<TIter> IntoPagedMemoryRegionIterator for TIter
where
    TIter: Iterator<Item = MemoryRegion>,
{
    fn into_paged_iter(self) -> PagedMemoryRegionIterator<Self> {
        PagedMemoryRegionIterator {
            iter: self.peekable(),
            current_page_start: 0,
            current_region_end: 0,
        }
    }
}

impl<TIter> Iterator for PagedMemoryRegionIterator<TIter>
where
    TIter: Iterator<Item = MemoryRegion>,
{
    type Item = MemoryRegionPage;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(next_region) = self.iter.peek()
                && (next_region.base_address <= self.current_region_end
                    || self.current_page_start >= self.current_region_end)
            {
                let next_region = self.iter.next()?;
                if self.current_page_start >= self.current_region_end {
                    self.current_page_start = align_up(next_region.base_address, LARGE_PAGE_SIZE);
                }
                self.current_region_end = next_region.base_address + next_region.length;
                continue;
            }

            assert_ne!(self.current_page_start, 0);

            if is_aligned(self.current_page_start, HUGE_PAGE_SIZE)
                && self.current_page_start + HUGE_PAGE_SIZE <= self.current_region_end
            {
                let current_page = self.current_page_start;
                self.current_page_start += HUGE_PAGE_SIZE;
                return Some(MemoryRegionPage::Huge(MemoryPage {
                    base_address: current_page,
                }));
            }

            assert!(is_aligned(self.current_page_start, LARGE_PAGE_SIZE));

            if self.current_page_start + LARGE_PAGE_SIZE <= self.current_region_end {
                let current_page = self.current_page_start;
                self.current_page_start += LARGE_PAGE_SIZE;
                return Some(MemoryRegionPage::Large(MemoryPage {
                    base_address: current_page,
                }));
            }

            let next_region = self.iter.next()?;
            self.current_page_start = align_up(next_region.base_address, LARGE_PAGE_SIZE);
            self.current_region_end = next_region.base_address + next_region.length;
        }
    }
}
