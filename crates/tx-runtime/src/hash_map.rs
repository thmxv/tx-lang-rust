use std::{alloc::Allocator, ptr::Unique};

pub trait HashMapKey<T> {
    const EMPTY_KEY: T;
}

pub trait HashMapValue<T> {
    const EMPTY_VALUE: T;
    const TOMBSTONE_VALUE: T;
}

pub struct HashMap<'a, KeyT, ValueT, A: Allocator>
where
    KeyT: HashMapKey<KeyT>,
    ValueT: HashMapValue<ValueT>,
{
    ptr: Unique<Entry<KeyT, ValueT>>,
    cap: usize,
    len: usize, // len includes tombstones
    #[cfg(debug_assertions)]
    allocator: &'a A,
}

pub struct Entry<KeyT, ValueT> {
    key: KeyT,
    value: ValueT,
}

impl<KeyT, ValueT, A: Allocator> HashMap<'_, KeyT, ValueT, A>
where
    KeyT: HashMapKey<KeyT>,
    ValueT: HashMapValue<ValueT>,
{
    pub(crate) const MAX_LOAD_FACTOR: f32 = 0.75;
}
