use core::alloc::{GlobalAlloc, Layout};

#[global_allocator]
static ALLOCATOR: LvglAlloc = LvglAlloc;

struct LvglAlloc;

unsafe impl GlobalAlloc for LvglAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        lvgl_sys::lv_mem_alloc(layout.size() as usize) as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        lvgl_sys::lv_mem_free(ptr as *mut cty::c_void)
    }

    unsafe fn realloc(&self, ptr: *mut u8, _layout: Layout, new_size: usize) -> *mut u8 {
        lvgl_sys::lv_mem_realloc(ptr as *mut cty::c_void, new_size) as *mut u8
    }
}

pub fn heap_init() {
    crate::core::ensure_init();
}
