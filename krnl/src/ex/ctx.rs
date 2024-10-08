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

use core::{arch, hint, ptr, slice};

use alloc::boxed::Box;

pub struct Context {
    _stack: Box<[u8]>,
    stack_ptr: *mut u8,
}

impl Context {
    pub unsafe fn empty() -> Self {
        Self {
            _stack: Box::new([]),
            stack_ptr: ptr::null_mut(),
        }
    }

    pub fn new(stack_size: usize, entry_point: fn() -> !) -> Self {
        // SAFETY: stack gets initialized as it is used
        let mut stack = unsafe {
            Box::from_raw(slice::from_raw_parts_mut(
                alloc::alloc::alloc(
                    core::alloc::Layout::from_size_align(stack_size, 4096).unwrap(),
                ),
                stack_size,
            ))
        };

        // SAFETY: stack is valid, large enough to encompass all element
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

        Self {
            _stack: stack,
            stack_ptr,
        }
    }

    /// The context is swapped by using the stack pointer specified by `self`.
    ///
    /// Note that this function cannot return as the previous stack pointer is
    /// not saved.
    pub fn load(&self) -> ! {
        // SAFETY:
        // - stack_ptr is guaranteed to be valid, as this is enforced by the `Context`
        //   struct;
        // - this function can never return because the current stack pointer is
        //   discarded.
        unsafe {
            context_swap(self.stack_ptr, &mut ptr::null_mut());
            hint::unreachable_unchecked()
        }
    }

    /// The context is swapped by using the stack pointer specified by `self`,
    /// and saving the previous one to `save`.
    pub fn swap(&self, save: &mut Self) {
        // SAFETY: Context guarantees stack_ptr to be valid
        unsafe {
            context_swap(self.stack_ptr, &mut save.stack_ptr);
        }
    }
}

#[naked]
unsafe extern "C" fn context_swap(load: *mut u8, save: &mut *mut u8) {
    // System V ABI for x86
    // - Arguments: stack
    // - Caller-saved: eax, ecx, edx
    // - Callee-saved: esp, ebp, ebx, esi, edi
    #[cfg(target_arch = "x86")]
    arch::naked_asm!(
        r#"
        mov eax, [esp + 0x04]
        mov edx, [esp + 0x08]
        push ebp
        push ebx
        push esi
        push edi
        mov [edx], esp

        mov esp, eax
        pop edi
        pop esi
        pop ebx
        pop ebp
        ret
        "#
    );

    // System V ABI for x86-64
    // - Arguments: rdi, rsi, rdx, rcx, r8, r9, stack
    // - Caller-saved: rax, rcx, rdx, rdi, rsi, r10, r11
    // - Callee-saved: rsp, rbp, rbx, r12, r13, r14, r15
    #[cfg(target_arch = "x86_64")]
    arch::naked_asm!(
        r#"
        push rbp
        push rbx
        push r12
        push r13
        push r14
        push r15
        mov [rsi], rsp

        mov rsp, rdi
        pop r15
        pop r14
        pop r13
        pop r12
        pop rbx
        pop rbp
        ret
        "#
    );
}
