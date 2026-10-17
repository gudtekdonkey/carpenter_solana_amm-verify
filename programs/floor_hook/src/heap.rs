//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!

use core::alloc::Layout;
use core::ptr::null_mut;

///
pub const HEAP_START: usize = 0x3_0000_0000;
///
pub const DEFAULT_HEAP_BYTES: usize = 32 * 1024;
///
pub const MAX_HEAP_BYTES: usize = 256 * 1024;

const WORD: usize = core::mem::size_of::<usize>();
///
pub const GRAIN: usize = 8;
///
///
#[cfg(not(feature = "heap-trace"))]
const HEADER: usize = WORD;
#[cfg(feature = "heap-trace")]
const HEADER: usize = 3 * WORD;

///
///
#[derive(Clone, Copy)]
pub struct Arena {
    base: usize,
    len: usize,
}

impl Arena {
///
///
///
    pub const unsafe fn new(base: usize, len: usize) -> Self {
        Self { base, len }
    }

    #[inline(always)]
    fn top_slot(&self) -> *mut usize {
        self.base as *mut usize
    }

///
    #[inline(always)]
    unsafe fn top(&self) -> usize {
        let t = *self.top_slot();
        if t == 0 {
            self.base + HEADER
        } else {
            t
        }
    }

    #[inline(always)]
    fn end(&self) -> usize {
        self.base + self.len
    }

///
///
///
    pub unsafe fn used(&self) -> usize {
        self.top() - self.base
    }

///
///
    #[inline]
    pub unsafe fn alloc(&self, layout: Layout) -> *mut u8 {

        let align = if layout.align() > GRAIN { layout.align() } else { GRAIN };
        let start = match self.top().checked_add(align - 1) {
            Some(v) => v & !(align - 1),
            None => return null_mut(),
        };
        match grained(layout.size()).and_then(|n| start.checked_add(n)) {
            Some(new_top) if new_top <= self.end() => {
                self.set_top(new_top, layout.size());
                start as *mut u8
            }
            _ => null_mut(),
        }
    }

///
    #[inline(always)]
    unsafe fn set_top(&self, new_top: usize, requested: usize) {
        *self.top_slot() = new_top;
        #[cfg(feature = "heap-trace")]
        {
            let peak = self.top_slot().add(1);
            if new_top > *peak {
                *peak = new_top;
            }
            *self.top_slot().add(2) += requested;
        }
        #[cfg(not(feature = "heap-trace"))]
        let _ = requested;
    }

///
///
///
///
///
    #[cfg(feature = "heap-trace")]
    pub unsafe fn trace(&self) -> (usize, usize) {
        let peak = core::cmp::max(*self.top_slot().add(1), self.top());
        (peak - self.base, *self.top_slot().add(2))
    }

///
///
///
    #[inline]
    pub unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {


        let p = ptr as usize;
        if cfg!(not(feature = "heap-no-reuse")) && p.wrapping_add(grained_live(layout.size())) == *self.top_slot() {
            *self.top_slot() = p;
        }
    }

///
///
///
    #[inline]
    pub unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let p = ptr as usize;
        if cfg!(not(feature = "heap-no-reuse")) && p.wrapping_add(grained_live(layout.size())) == *self.top_slot() {
            return match grained(new_size).and_then(|n| p.checked_add(n)) {
                Some(new_top) if new_top <= self.end() => {
                    self.set_top(new_top, new_size);
                    ptr
                }
                _ => null_mut(),
            };
        }
        let new_layout = Layout::from_size_align_unchecked(new_size, layout.align());
        let fresh = self.alloc(new_layout);
        if !fresh.is_null() {
            core::ptr::copy_nonoverlapping(ptr, fresh, core::cmp::min(layout.size(), new_size));
            self.dealloc(ptr, layout);
        }
        fresh
    }
}

///
#[inline(always)]
fn grained(n: usize) -> Option<usize> {
    n.checked_add(GRAIN - 1).map(|v| v & !(GRAIN - 1))
}

///
#[inline(always)]
fn grained_live(n: usize) -> usize {
    n.wrapping_add(GRAIN - 1) & !(GRAIN - 1)
}

///
///
#[cfg(all(target_os = "solana", feature = "custom-heap", not(feature = "no-entrypoint")))]
pub struct HookHeap;

#[cfg(all(target_os = "solana", feature = "custom-heap", not(feature = "no-entrypoint")))]
const REGION: Arena = unsafe { Arena::new(HEAP_START, MAX_HEAP_BYTES) };

#[cfg(all(target_os = "solana", feature = "custom-heap", not(feature = "no-entrypoint")))]
unsafe impl core::alloc::GlobalAlloc for HookHeap {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        REGION.alloc(layout)
    }
    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        REGION.dealloc(ptr, layout)
    }
    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        REGION.realloc(ptr, layout, new_size)
    }
}

#[cfg(all(target_os = "solana", feature = "custom-heap", not(feature = "no-entrypoint")))]
#[global_allocator]
static HEAP: HookHeap = HookHeap;

///
///
#[cfg(all(target_os = "solana", feature = "custom-heap", feature = "heap-trace", not(feature = "no-entrypoint")))]
pub fn trace() -> (usize, usize, usize) {
    unsafe {
        let (peak, requested) = REGION.trace();
        (REGION.used(), peak, requested)
    }
}

#[cfg(test)]
mod tests {
//!
    use super::*;

///
    fn arena(len: usize) -> (Vec<u128>, Arena) {
        let buf = vec![0u128; len / 16];
        let a = unsafe { Arena::new(buf.as_ptr() as usize, len) };
        (buf, a)
    }

    fn l(size: usize, align: usize) -> Layout {
        Layout::from_size_align(size, align).unwrap()
    }

    #[test]
    fn it_bumps_upward_from_one_word_past_the_start() {
        let (_b, a) = arena(1024);
        unsafe {
            let p = a.alloc(l(10, 1)) as usize;
            assert_eq!(p, a.base + HEADER, "the first block sits just past the header");
            let q = a.alloc(l(8, 8)) as usize;
            assert_eq!(q, p + 16, "the 10-byte block takes 16: every block ENDS on the grain");
            assert_eq!(a.used(), q + 8 - a.base);
            let r = a.alloc(l(3, 32)) as usize;
            assert_eq!(r % 32, 0, "a larger alignment is honoured");
        }
    }

    #[test]
    fn the_last_block_freed_is_handed_back_and_no_other() {
        let (_b, a) = arena(1024);
        unsafe {
            let x = a.alloc(l(100, 1));
            let y = a.alloc(l(50, 1));
            let before = a.used();
            a.dealloc(x, l(100, 1));
            assert_eq!(a.used(), before);
            a.dealloc(y, l(50, 1));
            assert_eq!(a.used(), y as usize - a.base);
            let z = a.alloc(l(50, 1));
            assert_eq!(z, y, "y's bytes are reused");
        }
    }

///
///
    #[test]
    fn freed_in_reverse_order_a_whole_cpi_costs_nothing() {

        let (_b, a) = arena(4096);
        unsafe {
            let base = a.used();
            let data = a.alloc(l(92, 1));
            let metas = a.alloc(l(21 * 34, 1));
            let infos = a.alloc(l(19 * 48, 8));
            let ret = a.alloc(l(150, 1));
            a.dealloc(ret, l(150, 1));
            a.dealloc(infos, l(19 * 48, 8));
            a.dealloc(metas, l(21 * 34, 1));
            a.dealloc(data, l(92, 1));
            assert_eq!(a.used(), base, "a CPI leaves the heap where it found it");
        }
    }

    #[test]
    fn the_last_block_grows_and_shrinks_in_place_any_other_moves() {
        let (_b, a) = arena(1024);
        unsafe {
            let v = a.alloc(l(16, 8));
            *(v as *mut u64) = 0xDEAD_BEEF;
            let v2 = a.realloc(v, l(16, 8), 64);
            assert_eq!(v2, v, "the last block grows in place");
            assert_eq!(a.used(), v as usize + 64 - a.base);
            let v3 = a.realloc(v2, l(64, 8), 24);
            assert_eq!(v3, v, "and shrinks in place");
            assert_eq!(a.used(), v as usize + 24 - a.base);
            let w = a.alloc(l(8, 8));
            let v4 = a.realloc(v3, l(24, 8), 200);
            assert!(v4 as usize > w as usize, "a block that is not last moves above the top");
            assert_eq!(*(v4 as *mut u64), 0xDEAD_BEEF, "and keeps its bytes");
        }
    }

    #[test]
    fn past_the_region_it_refuses_and_changes_nothing() {
        let (_b, a) = arena(256);
        unsafe {
            let used = a.used();
            let rest = 256 - HEADER;
            assert!(a.alloc(l(rest + 1, 1)).is_null(), "the header's words are not handed out");
            assert_eq!(a.used(), used, "a refused allocation moves nothing");
            let p = a.alloc(l(rest, 1));
            assert!(!p.is_null(), "exactly the rest fits");
            assert!(a.alloc(l(1, 1)).is_null());
            assert!(a.realloc(p, l(rest, 1), rest + 1).is_null(), "the last block cannot grow past the end");
            assert_eq!(a.used(), 256);
            assert!(a.alloc(l(isize::MAX as usize, 1)).is_null(), "the largest legal size is refused, never wrapped");
        }
    }

    #[test]
    fn the_runtime_constants_are_the_runtimes() {
        assert_eq!(HEAP_START as u64, anchor_lang::solana_program::entrypoint::HEAP_START_ADDRESS);
        assert_eq!(DEFAULT_HEAP_BYTES, anchor_lang::solana_program::entrypoint::HEAP_LENGTH);
        assert_eq!(MAX_HEAP_BYTES, 256 * 1024);
        assert_eq!(MAX_HEAP_BYTES % 1024, 0);
    }
}
