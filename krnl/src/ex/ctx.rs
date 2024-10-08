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

use core::{hint, ptr};

use alloc::boxed::Box;

pub struct Context {
    stack: Box<[u8]>,
    stack_ptr: *mut u8,
}

impl Context {
    pub unsafe fn empty() -> Self {
        Self {
            stack: Box::new([]),
            stack_ptr: ptr::null_mut(),
        }
    }

    pub fn new(entry_point: fn() -> !) -> Self {
        let mut stack = unsafe { Box::<[u8; 8 * 1024]>::new_uninit().assume_init() };
        let stack_ptr = unsafe {
            let mut stack_ptr = stack.as_mut_ptr() as *mut usize;
            stack_ptr = stack_ptr.sub(1); // eip/rip
            stack_ptr.write(entry_point as usize);
            #[cfg(target_arch = "x86")]
            {
                stack_ptr = stack_ptr.sub(4); // ebx, ebp, esi, edi
            }
            #[cfg(target_arch = "x86_64")]
            {
                stack_ptr = stack_ptr.sub(6); // rbx, rbp, r12, r13, r14, r15
            }
            stack_ptr as *mut u8
        };
        Self { stack, stack_ptr }
    }

    pub fn load(&self) -> ! {
        unsafe {
            context_swap(self.stack_ptr, &mut ptr::null_mut());
            hint::unreachable_unchecked()
        }
    }

    pub fn swap(&self, save: &mut Self) {
        unsafe {
            context_swap(self.stack_ptr, &mut save.stack_ptr);
        }
    }
}

extern "C" {
    fn context_swap(load: *mut u8, save: &mut *mut u8);
}
