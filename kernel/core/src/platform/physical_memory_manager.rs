use crate::platform::memory_layout::PAGE_FRAME_SIZE;
use crate::platform::physical_page_frame::{PhysicalPageFrame, PhysicalPageFrameParseError};
use crate::println;
use core::ptr::NonNull;
use kernel_bindings_gen::{LIMINE_MEMMAP_USABLE, limine_memmap_entry, limine_memmap_response};
use nom::multi::many_till;

pub struct PhysicalMemoryManager;

#[derive(Debug)]
pub enum PhysicalMemoryManagerAllocFrameError {
    NoAvailablePages,
    InvalidPageFrame(PhysicalPageFrameParseError),
}

impl From<PhysicalPageFrameParseError> for PhysicalMemoryManagerAllocFrameError {
    fn from(err: PhysicalPageFrameParseError) -> Self {
        Self::InvalidPageFrame(err)
    }
}

fn align_up(v: usize, a: usize) -> usize {
    let mask = !(a - 1);
    (v + (a - 1)) & mask
}

fn align_down(v: usize, a: usize) -> usize {
    let mask = !(a - 1);
    v & mask
}

const STACK_CAPACITY: usize = 0x100_000; // up to 1024^2 page frames
static mut PHYSICAL_PAGE_FRAMES: [usize; STACK_CAPACITY] = [0; STACK_CAPACITY];
static mut PAGE_FRAMES_COUNT: usize = 0;

unsafe fn push_frame(frame: usize) {
    unsafe {
        if STACK_CAPACITY <= PAGE_FRAMES_COUNT {
            return;
        }

        PHYSICAL_PAGE_FRAMES[PAGE_FRAMES_COUNT] = frame;
        PAGE_FRAMES_COUNT += 1;
    }
}

unsafe fn pop_frame() -> usize {
    unsafe {
        if PAGE_FRAMES_COUNT == 0 {
            return 0;
        }

        PAGE_FRAMES_COUNT -= 1;
        PHYSICAL_PAGE_FRAMES[PAGE_FRAMES_COUNT]
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn pmm_init(memmap: *mut limine_memmap_response) {
    let memmap = NonNull::new(memmap).expect("Memmap response not provided!");

    let entries = unsafe {
        let memmap = memmap.as_ref();
        let base = memmap.entries;
        let count = memmap.entry_count as usize;
        core::slice::from_raw_parts::<*mut limine_memmap_entry>(base, count)
    };

    for entry in entries.into_iter() {
        let entry = match NonNull::new(*entry) {
            None => continue,
            Some(entry) => unsafe { entry.as_ref() },
        };
        if entry.type_ != LIMINE_MEMMAP_USABLE as u64 {
            continue;
        }

        let base = entry.base as usize;
        let len = entry.length as usize;

        let start = align_up(base, PAGE_FRAME_SIZE);
        let end = align_down(base + len, PAGE_FRAME_SIZE);

        if start >= end {
            continue;
        }

        let pages_in_chunk = (end - start) / PAGE_FRAME_SIZE;
        for i in 0..pages_in_chunk {
            unsafe {
                push_frame(start + i * PAGE_FRAME_SIZE);
            }
        }
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn pmm_alloc_frame() -> usize {
    unsafe { pop_frame() }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn pmm_free_frame(frame: usize) -> bool {
    unsafe {
        if frame == 0 {
            panic!("Attempted to free null page frame!");
        }

        if frame % PAGE_FRAME_SIZE != 0 {
            panic!("Attempted to free non-page-aligned page frame!");
        }

        push_frame(frame);
        true
    }
}

impl PhysicalMemoryManager {
    pub unsafe fn init(memmap: *mut limine_memmap_response) {
        unsafe {
            pmm_init(memmap);
        }
    }

    pub unsafe fn alloc_frame() -> Result<PhysicalPageFrame, PhysicalMemoryManagerAllocFrameError> {
        let page_frame = unsafe { pmm_alloc_frame() };
        if page_frame == 0 {
            return Err(PhysicalMemoryManagerAllocFrameError::NoAvailablePages);
        }

        Ok(PhysicalPageFrame::new(page_frame)?)
    }
}
