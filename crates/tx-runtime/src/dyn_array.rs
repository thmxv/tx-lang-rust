use std::{
    alloc::{handle_alloc_error, Allocator, Layout},
    cmp,
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut},
    ptr::{self, Unique},
};

pub struct RawDynArray<'a, T, A: Allocator> {
    ptr: Unique<T>,
    cap: usize,
    #[cfg(debug_assertions)]
    allocator: &'a A,
}

impl<'a, T, A: Allocator> RawDynArray<'a, T, A> {
    // Tiny Vecs are dumb. Skip to:
    // - 8 if the element size is 1, because any heap allocators is likely
    //   to round up a request of less than 8 bytes to at least 8 bytes.
    // - 4 if elements are moderate-sized (<= 1 KiB).
    // - 1 otherwise, to avoid wasting too much space for very short Vecs.
    pub(crate) const MIN_NON_ZERO_CAP: usize = if mem::size_of::<T>() == 1 {
        8
    } else if mem::size_of::<T>() <= 1024 {
        4
    } else {
        1
    };

    fn new(alloc: & 'a A) -> Self {
        Self {
            ptr: Unique::dangling(),
            cap: if mem::size_of::<T>() == 0 {
                usize::MAX
            } else {
                0
            },
            #[cfg(debug_assertions)]
            allocator: alloc,
        }
    }

    #[inline]
    pub unsafe fn reserve(
        &mut self,
        alloc: &A,
        len: usize,
        additional: usize,
    ) {
        debug_assert!(ptr::eq(alloc, self.allocator));
        // Callers expect this function to be very cheap when there is already
        // sufficient capacity. Therefore, we move all the resizing and
        // error-handling logic from grow_amortized and handle_reserve behind
        // a call, while making sure that this function is likely to be
        // inlined as just a comparison and a call if the comparison fails.
        #[cold]
        fn do_reserve_and_handle<T, A: Allocator>(
            slf: &mut RawDynArray<T, A>,
            alloc: &A,
            len: usize,
            additional: usize,
        ) {
            unsafe { slf.grow(alloc, len, additional); }
        }

        if additional > self.cap.wrapping_sub(len) {
            do_reserve_and_handle(self, alloc, len, additional);
        }
    }

    unsafe fn grow(&mut self, alloc: &A, len: usize, additional: usize) {
        debug_assert!(ptr::eq(alloc, self.allocator));
        // since we set the capacity to usize::MAX when T has size 0,
        // getting to here necessarily means the Vec is overfull.
        assert!(mem::size_of::<T>() != 0, "capacity overflow");
        let required_cap = len + additional;
        // This can't overflow because we ensure self.cap <= isize::MAX.
        let new_cap = cmp::max(2 * self.cap, required_cap);
        let new_cap = cmp::max(Self::MIN_NON_ZERO_CAP, new_cap);
        // `Layout::array` checks that the number of bytes is <= usize::MAX,
        // but this is redundant since old_layout.size() <= isize::MAX,
        // so the `unwrap` should never fail.
        let new_layout = Layout::array::<T>(new_cap).unwrap();
        // Ensure that the new allocation doesn't exceed `isize::MAX` bytes.
        assert!(
            new_layout.size() <= isize::MAX as usize,
            "Allocation too large"
        );
        let new_ptr = if self.cap == 0 {
            alloc.allocate(new_layout)
        } else {
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            unsafe {
                alloc.grow(self.ptr.cast().into(), old_layout, new_layout)
            }
        };
        self.ptr = match new_ptr {
            Ok(p) => unsafe { Unique::new_unchecked(p.cast().as_ptr()) },
            Err(_) => handle_alloc_error(new_layout),
        };
        self.cap = new_cap;
    }

    unsafe fn destroy(&mut self, alloc: &A) {
        debug_assert!(ptr::eq(alloc, self.allocator));
        let elem_size = mem::size_of::<T>();
        if self.cap != 0 && elem_size != 0 {
            let layout = Layout::array::<T>(self.cap).unwrap();
            unsafe {
                alloc.deallocate(self.ptr.cast().into(), layout);
            }
        }
    }
}

#[cfg(debug_assertions)]
impl<T, A: Allocator> Drop for RawDynArray<'_, T, A> {
    fn drop(&mut self) {
        if mem::size_of::<T>() != 0 {
            debug_assert_eq!(self.cap, 0);
        }
    }
}

pub struct DynArray<'a, T, A: Allocator> {
    buf: RawDynArray<'a, T, A>,
    len: usize,
}

impl<'a, T, A: Allocator> DynArray<'a, T, A> {
    fn ptr(&self) -> *mut T {
        self.buf.ptr.as_ptr()
    }

    fn cap(&self) -> usize {
        self.buf.cap
    }

    pub fn new(alloc: & 'a A) -> Self {
        DynArray {
            buf: RawDynArray::new(alloc),
            len: 0,
        }
    }

    pub unsafe fn destroy(&mut self, alloc: &A) {
        while let Some(_) = self.pop() {}
        self.buf.destroy(alloc);
    }

    pub unsafe fn reserve(&mut self, alloc: &A, additional: usize) {
        self.buf.reserve(alloc, self.len, additional)
    }

    pub unsafe fn push(&mut self, alloc: &A, elem: T) {
        self.buf.reserve(alloc, self.len, 1);
        unsafe {
            ptr::write(self.ptr().add(self.len), elem);
        }
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe { Some(ptr::read(self.ptr().add(self.len))) }
        }
    }

    pub unsafe fn insert(&mut self, alloc: &A, index: usize, elem: T) {
        assert!(index <= self.len, "index out of bounds");
        self.buf.reserve(alloc, self.len, 1);
        unsafe {
            ptr::copy(
                self.ptr().add(index),
                self.ptr().add(index + 1),
                self.len - index,
            );
            ptr::write(self.ptr().add(index), elem);
            self.len += 1;
        }
    }

    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len, "index out of bounds");
        unsafe {
            self.len -= 1;
            let result = ptr::read(self.ptr().add(index));
            ptr::copy(
                self.ptr().add(index + 1),
                self.ptr().add(index),
                self.len - index,
            );
            result
        }
    }
}

impl<'a, T: Clone, A: Allocator> DynArray<'a, T, A> {
    pub unsafe fn resize(&mut self, alloc: &A, new_len: usize, value: T) {
        if self.len < new_len {
            self.reserve(alloc, new_len.wrapping_sub(self.len));
            while self.len < new_len {
                self.push(alloc, value.clone());
            }
        }
        while self.len > new_len {
            self.pop();
        }
    }
}

impl<T, A: Allocator> Deref for DynArray<'_, T, A> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.ptr(), self.len) }
    }
}

impl<T, A: Allocator> DerefMut for DynArray<'_, T, A> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr(), self.len) }
    }
}

impl<'a, T, A: Allocator> IntoIterator for DynArray<'a, T, A> {
    type Item = T;
    type IntoIter = IntoIter<'a, T, A>;
    fn into_iter(self) -> IntoIter<'a, T, A> {
        unsafe {
            let iter = RawValIter::new(&self);
            let buf = ptr::read(&self.buf);
            mem::forget(self);
            IntoIter { iter, _buf: buf }
        }
    }
}

struct RawValIter<T> {
    start: *const T,
    end: *const T,
}

impl<T> RawValIter<T> {
    // unsafe to construct because it has no associated lifetimes.
    // This is necessary to store a RawValIter in the same struct as
    // its actual allocation. OK since it's a private implementation
    // detail.
    unsafe fn new(slice: &[T]) -> Self {
        RawValIter {
            start: slice.as_ptr(),
            end: if mem::size_of::<T>() == 0 {
                ((slice.as_ptr() as usize) + slice.len()) as *const _
            } else if slice.len() == 0 {
                // if `len = 0`, then this is not actually allocated memory.
                // Need to avoid offsetting because that will give wrong
                // information to LLVM via GEP.
                slice.as_ptr()
            } else {
                slice.as_ptr().add(slice.len())
            },
        }
    }
}

impl<T> Iterator for RawValIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                if mem::size_of::<T>() == 0 {
                    self.start = (self.start as usize + 1) as *const _;
                    Some(ptr::read(Unique::<T>::dangling().as_ptr()))
                } else {
                    let old_ptr = self.start;
                    self.start = self.start.offset(1);
                    Some(ptr::read(old_ptr))
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let elem_size = mem::size_of::<T>();
        let len = (self.end as usize - self.start as usize)
            / if elem_size == 0 { 1 } else { elem_size };
        (len, Some(len))
    }
}

impl<T> DoubleEndedIterator for RawValIter<T> {
    fn next_back(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                if mem::size_of::<T>() == 0 {
                    self.end = (self.end as usize - 1) as *const _;
                    Some(ptr::read(Unique::<T>::dangling().as_ptr()))
                } else {
                    self.end = self.end.offset(-1);
                    Some(ptr::read(self.end))
                }
            }
        }
    }
}

pub struct IntoIter<'a, T, A: Allocator> {
    _buf: RawDynArray<'a, T, A>, // Just need it to stay alive
    iter: RawValIter<T>,
}

impl<T, A: Allocator> IntoIter<'_, T, A> {
    pub unsafe fn destroy(&mut self, alloc: &A) {
        for _ in &mut *self {}
        self._buf.destroy(alloc);
    }
}

impl<T, A: Allocator> Iterator for IntoIter<'_, T, A> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T, A: Allocator> DoubleEndedIterator for IntoIter<'_, T, A> {
    fn next_back(&mut self) -> Option<T> {
        self.iter.next_back()
    }
}

pub struct Drain<'a, T: 'a> {
    vec: PhantomData<&'a mut Vec<T>>,
    iter: RawValIter<T>,
}

impl<'a, T> Iterator for Drain<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.iter.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T> DoubleEndedIterator for Drain<'a, T> {
    fn next_back(&mut self) -> Option<T> {
        self.iter.next_back()
    }
}

impl<'a, T> Drop for Drain<'a, T> {
    fn drop(&mut self) {
        // pre-drain the iter
        for _ in &mut *self {}
    }
}
