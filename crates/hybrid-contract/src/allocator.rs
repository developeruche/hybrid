//! # Hybrid Bump Allocator
//!
//! This module implements a simple bump allocator for use in a RISCV environment.
//! It provides a custom global allocator for Rust, managing a fixed-size heap
//! using a bump pointer strategy. This allocator is suitable for environments
//! where memory is allocated but not freed (no deallocation).
//!
//! ## Features
//! - Fixed-size heap (10 MiB)
//! - Simple bump allocation (no reuse of freed memory)
//! - Alignment support
//! - Panic on out-of-memory
//!
//! ## Safety
//! This allocator uses unsafe code and static mutable state. It is intended for
//! single-threaded or bare-metal environments where such usage is acceptable.

#![allow(static_mut_refs)]
use core::alloc::{GlobalAlloc, Layout};

/// The main bump allocator instance.
/// This is initialized on first allocation.
static mut MAIN_ALLOC: Option<MainAlloc> = None;

/// The total heap size in bytes (10 MiB).
static HEAP_WIDTH: usize = 1024 * 1024 * 10;

/// The backing heap storage.
static mut HEAP: [u8; HEAP_WIDTH] = [0; HEAP_WIDTH];

/// Internal structure managing the bump pointer and heap bounds.
struct MainAlloc {
    /// The next available allocation address.
    next: usize,
    /// The maximum heap address (end of heap).
    max: usize,
}

impl MainAlloc {
    /// Creates a new `MainAlloc` instance, initializing the bump pointer and heap bounds.
    fn new() -> Self {
        MainAlloc {
            next: Self::heap_start(),
            max: Self::heap_end(),
        }
    }

    /// Returns the start address of the heap.
    fn heap_start() -> usize {
        unsafe { HEAP.as_mut_ptr() as usize }
    }

    /// Returns the end address of the heap.
    fn heap_end() -> usize {
        Self::heap_start() + HEAP_WIDTH
    }

    /// Allocates memory with the given layout, returning the start address.
    ///
    /// # Safety
    /// Caller must ensure exclusive access to the allocator.
    unsafe fn alloc(&mut self, layout: Layout) -> Option<usize> {
        let alloc_start = self.align_ptr(&layout);
        let size = layout.size();
        let end = alloc_start.checked_add(size)?;
        if end > self.max {
            panic!("HybridAlloc: Out of memory");
        } else {
            self.next = end;
            Some(alloc_start)
        }
    }

    /// Aligns the bump pointer according to the layout's alignment.
    fn align_ptr(&self, layout: &Layout) -> usize {
        (self.next + layout.align() - 1) & !(layout.align() - 1)
    }
}

/// Global bump allocator for RISCV machines.
///
/// Implements the `GlobalAlloc` trait for use as Rust's global allocator.
pub struct HybridAlloc;

unsafe impl GlobalAlloc for HybridAlloc {
    /// Allocates memory with the given layout.
    ///
    /// # Safety
    /// This function is unsafe because it may access static mutable state.
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if MAIN_ALLOC.is_none() {
            MAIN_ALLOC = Some(MainAlloc::new());
        }

        if let Some(start) = MAIN_ALLOC.as_mut().unwrap().alloc(layout) {
            start as *mut u8
        } else {
            core::ptr::null_mut()
        }
    }

    /// Allocates zeroed memory with the given layout.
    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        self.alloc(layout)
    }

    /// Deallocation is a no-op for bump allocators.
    #[inline]
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

/// Sets `HybridAlloc` as the global allocator.
#[global_allocator]
static mut ALLOC: HybridAlloc = HybridAlloc {};
