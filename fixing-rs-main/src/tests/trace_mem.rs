use backtrace::Backtrace;
use std::alloc::{GlobalAlloc, Layout, System};

#[global_allocator]
static ALLOC: Alloc = Alloc;

struct Alloc;

const TRACE_MIN_SIZE: usize = 128 * 1024 * 1024;

fn check_alloc_size(layout: &Layout, realloc: bool) {
    if layout.size() >= TRACE_MIN_SIZE {
        log::warn!(
            "{}alloc: {} bytes.\n{:?}",
            if realloc { "re" } else { "" },
            layout.size(),
            Backtrace::new()
        );
    }
}

unsafe impl GlobalAlloc for Alloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        check_alloc_size(&layout, false);
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        check_alloc_size(&layout, false);
        System.alloc_zeroed(layout)
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        check_alloc_size(&layout, true);
        System.realloc(ptr, layout, new_size)
    }
}
