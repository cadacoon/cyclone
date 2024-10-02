// Copyright 2024 Kevin Ludwig
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use core::{mem, ptr};

#[repr(transparent)]
pub struct Context(Option<ptr::NonNull<()>>);

unsafe impl Sync for Context {}

unsafe impl Send for Context {}

impl Context {
    pub const fn zeroed() -> Self {
        Self(None)
    }

    pub unsafe fn new(entry_point: fn() -> !, stack_base: *mut (), stack_size: usize) -> Self {
        let mut stack = stack_base.byte_add(stack_size) as *mut usize;
        stack = stack.sub(1); // eip/rip
        stack.write(entry_point as usize);
        #[cfg(target_arch = "x86")]
        {
            stack = stack.sub(4); // ebx, ebp, esi, edi
        }
        #[cfg(target_arch = "x86_64")]
        {
            stack = stack.sub(6); // rbx, rbp, r12, r13, r14, r15
        }
        Self(Some(ptr::NonNull::new_unchecked(stack as *mut ())))
    }

    pub fn swap(current: &mut Self, target: &Self) {
        unsafe {
            __context_swap(mem::transmute(current), target.0.unwrap().as_ptr());
        }
    }
}

extern "C" {
    fn __context_swap(current: &mut *mut (), target: *mut ());
}
