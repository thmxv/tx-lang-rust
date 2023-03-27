use std::alloc::{AllocError, Allocator, Layout};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
pub struct Alloc<A: Allocator> {
    inner: A,
    allocated_bytes: AtomicUsize,
}

impl<A: Allocator> Alloc<A> {
    const fn new(inner: A) -> Self {
        Self {
            inner,
            allocated_bytes: AtomicUsize::new(0),
        }
    }

    pub fn allocated_bytes(&self) -> usize {
        self.allocated_bytes.load(Ordering::Relaxed)
    }
}

unsafe impl<A: Allocator> Allocator for Alloc<A> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.allocated_bytes
            .fetch_add(layout.size(), Ordering::Relaxed);
        self.inner.allocate(layout)
    }

    fn allocate_zeroed(
        &self,
        layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        self.allocated_bytes
            .fetch_add(layout.size(), Ordering::Relaxed);
        self.inner.allocate_zeroed(layout)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        self.allocated_bytes
            .fetch_sub(layout.size(), Ordering::Relaxed);
        self.inner.deallocate(ptr, layout)
    }

    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        self.allocated_bytes.fetch_add(
            new_layout.size().wrapping_sub(old_layout.size()),
            Ordering::Relaxed,
        );
        self.inner.grow(ptr, old_layout, new_layout)
    }

    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        self.allocated_bytes.fetch_add(
            new_layout.size().wrapping_sub(old_layout.size()),
            Ordering::Relaxed,
        );
        self.inner.grow_zeroed(ptr, old_layout, new_layout)
    }

    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        self.allocated_bytes.fetch_sub(
            old_layout.size().wrapping_sub(new_layout.size()),
            Ordering::Relaxed,
        );
        self.inner.shrink(ptr, old_layout, new_layout)
    }
}
