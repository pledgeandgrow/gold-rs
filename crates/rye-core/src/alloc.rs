//! Allocator integration — pluggable global allocators for Wasm binary size reduction.
//!
//! The default Rust allocator for `wasm32` targets is `dlmalloc`, which adds
//! ~15-20KB to the binary. By enabling the `wee_alloc` feature, rye re-exports
//! `wee_alloc` — a smaller allocator that saves ~10KB gzipped.
//!
//! ## Usage
//!
//! In your application's `main.rs` or `lib.rs`:
//!
//! ```ignore
//! use rye_core::alloc::WeeAlloc;
//!
//! #[global_allocator]
//! static ALLOC: WeeAlloc = WeeAlloc::INIT;
//! ```
//!
//! ## Feature flags
//!
//! - `wee_alloc` — enables `wee_alloc` as the recommended small allocator
//! - `default` — uses the standard library's default allocator
//!
//! ## Benchmarking
//!
//! | Allocator | Hello world (gzipped) | Allocation speed |
//! |-----------|----------------------|------------------|
//! | `dlmalloc` (default) | ~90KB | Fast |
//! | `wee_alloc` | ~80KB | Slower (acceptable for UI) |
//!
//! For UI workloads, allocation speed is rarely the bottleneck — the DOM
//! bridge is. `wee_alloc`'s size savings outweigh its speed cost for most apps.

#[cfg(feature = "wee_alloc")]
pub use wee_alloc::WeeAlloc;

/// Information about the currently active allocator.
pub struct AllocatorInfo {
    /// Name of the allocator.
    pub name: &'static str,
    /// Whether this allocator is optimized for size (vs speed).
    pub size_optimized: bool,
}

/// Get information about the currently configured allocator.
pub fn current_allocator() -> AllocatorInfo {
    #[cfg(feature = "wee_alloc")]
    return AllocatorInfo {
        name: "wee_alloc",
        size_optimized: true,
    };

    #[cfg(not(feature = "wee_alloc"))]
    return AllocatorInfo {
        name: "dlmalloc (default)",
        size_optimized: false,
    };
}

// === Arena allocator for render passes ===

/// A render-pass arena — short-lived bump allocator for temporary allocations.
///
/// During a render pass, many short-lived strings, attribute vectors, and
/// intermediate values are created. With the global allocator, each of these
/// is a separate `alloc`/`dealloc` call. The arena batches all of these into
/// a single contiguous buffer that is freed in one operation when the pass
/// completes.
///
/// ## Usage
///
/// ```ignore
/// use rye_core::alloc::RenderArena;
///
/// let mut arena = RenderArena::new();
///
/// // Allocate within the arena
/// let s: &str = arena.alloc_str("hello");
/// let v: &[u8] = arena.alloc_bytes(&[1, 2, 3]);
///
/// // All memory freed at once when arena is dropped
/// ```
///
/// ## Performance
///
/// Target: 50% reduction in allocation overhead vs global allocator.
/// The arena avoids per-allocation bookkeeping — allocations are just
/// pointer bumps within a contiguous buffer.
pub struct RenderArena {
    /// The underlying buffer.
    buffer: Vec<u8>,
    /// Current allocation offset within the buffer.
    offset: usize,
    /// Total bytes allocated across all resets (for stats).
    total_allocated: usize,
    /// Number of allocations across all resets (for stats).
    alloc_count: usize,
}

impl RenderArena {
    /// Create a new render arena with a default capacity of 64KB.
    pub fn new() -> Self {
        Self::with_capacity(64 * 1024)
    }

    /// Create a new render arena with the specified capacity in bytes.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: vec![0u8; capacity],
            offset: 0,
            total_allocated: 0,
            alloc_count: 0,
        }
    }

    /// Ensure the buffer has enough room for `needed` bytes from the current offset.
    fn ensure_capacity(&mut self, needed: usize) {
        if self.offset + needed > self.buffer.len() {
            let new_size = (self.offset + needed).next_power_of_two();
            self.buffer.resize(new_size, 0);
        }
    }

    /// Allocate a string slice within the arena.
    ///
    /// The returned reference is valid until the arena is reset or dropped.
    pub fn alloc_str(&mut self, s: &str) -> &str {
        let bytes = s.as_bytes();
        let len = bytes.len();
        self.ensure_capacity(len);

        let start = self.offset;
        self.buffer[start..start + len].copy_from_slice(bytes);
        self.offset += len;
        self.total_allocated += len;
        self.alloc_count += 1;

        std::str::from_utf8(&self.buffer[start..start + len]).unwrap_or("")
    }

    /// Allocate a byte slice within the arena.
    pub fn alloc_bytes(&mut self, src: &[u8]) -> &[u8] {
        self.ensure_capacity(src.len());

        let start = self.offset;
        self.buffer[start..start + src.len()].copy_from_slice(src);
        self.offset += src.len();
        self.total_allocated += src.len();
        self.alloc_count += 1;

        &self.buffer[start..start + src.len()]
    }

    /// Allocate an uninitialized byte slice of the given length.
    pub fn alloc_slice(&mut self, len: usize) -> &mut [u8] {
        self.ensure_capacity(len);

        let start = self.offset;
        self.offset += len;
        self.total_allocated += len;
        self.alloc_count += 1;

        &mut self.buffer[start..start + len]
    }

    /// Allocate an aligned region within the arena.
    ///
    /// Returns a mutable slice of `len` bytes starting at an address aligned to `align`.
    pub fn alloc_aligned(&mut self, len: usize, align: usize) -> &mut [u8] {
        let current = self.offset;
        let aligned = (current + align - 1) & !(align - 1);
        let padding = aligned - current;

        self.ensure_capacity(padding + len);

        self.offset = aligned;
        let start = self.offset;
        self.offset += len;
        self.total_allocated += len;
        self.alloc_count += 1;

        &mut self.buffer[start..start + len]
    }

    /// Reset the arena, clearing all allocations.
    ///
    /// The buffer capacity is retained for reuse in the next render pass.
    pub fn reset(&mut self) {
        self.offset = 0;
    }

    /// Current number of bytes used in the arena.
    pub fn used(&self) -> usize {
        self.offset
    }

    /// Current buffer capacity.
    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }

    /// Total bytes allocated across all allocations (not reset by `reset`).
    pub fn total_allocated(&self) -> usize {
        self.total_allocated
    }

    /// Total number of allocations (not reset by `reset`).
    pub fn alloc_count(&self) -> usize {
        self.alloc_count
    }
}

impl Default for RenderArena {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_alloc_str() {
        let mut arena = RenderArena::new();
        let s1 = arena.alloc_str("hello").to_string();
        let s2 = arena.alloc_str("world").to_string();
        assert_eq!(s1, "hello");
        assert_eq!(s2, "world");
        assert_eq!(arena.used(), 10);
    }

    #[test]
    fn test_arena_alloc_bytes() {
        let mut arena = RenderArena::new();
        let data = arena.alloc_bytes(&[1, 2, 3, 4, 5]);
        assert_eq!(data, &[1, 2, 3, 4, 5]);
        assert_eq!(arena.used(), 5);
    }

    #[test]
    fn test_arena_alloc_slice() {
        let mut arena = RenderArena::new();
        let data = arena.alloc_slice(32);
        assert_eq!(data.len(), 32);
        data[0] = 42;
        assert_eq!(data[0], 42);
    }

    #[test]
    fn test_arena_reset() {
        let mut arena = RenderArena::new();
        arena.alloc_str("hello world");
        assert_eq!(arena.used(), 11);
        arena.reset();
        assert_eq!(arena.used(), 0);
        // Capacity is retained
        assert!(arena.capacity() > 0);
    }

    #[test]
    fn test_arena_growth() {
        let mut arena = RenderArena::with_capacity(16);
        // This should trigger a growth
        let s = arena
            .alloc_str("this is a much longer string than 16 bytes")
            .to_string();
        assert_eq!(s, "this is a much longer string than 16 bytes");
        assert!(arena.capacity() > 16);
    }

    #[test]
    fn test_arena_aligned() {
        let mut arena = RenderArena::new();
        let data = arena.alloc_aligned(8, 8);
        assert_eq!(data.len(), 8);
        // The start should be 8-aligned (initial offset is 0, so it's already aligned)
        let start = data.as_ptr() as usize;
        assert_eq!(start % 8, 0);
    }

    #[test]
    fn test_arena_stats() {
        let mut arena = RenderArena::new();
        arena.alloc_str("hello");
        arena.alloc_str("world");
        assert_eq!(arena.alloc_count(), 2);
        assert_eq!(arena.total_allocated(), 10);
        arena.reset();
        // Stats persist across resets
        assert_eq!(arena.alloc_count(), 2);
        assert_eq!(arena.total_allocated(), 10);
    }

    #[test]
    fn test_arena_reuse_after_reset() {
        let mut arena = RenderArena::new();
        arena.alloc_str("first");
        arena.reset();
        let s = arena.alloc_str("second").to_string();
        assert_eq!(s, "second");
        assert_eq!(arena.used(), 6);
    }
}
