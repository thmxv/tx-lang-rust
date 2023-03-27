use std::alloc::Global;

use crate::allocator::Alloc;

type InnerAlloc = Global;
pub type VmAlloc = Alloc<InnerAlloc>;

pub struct VM {
    pub allocator: VmAlloc,
}


