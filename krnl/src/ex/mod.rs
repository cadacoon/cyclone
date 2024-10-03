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

use core::{arch, hint, ptr};

use alloc::{boxed::Box, collections::vec_deque::VecDeque};
use tracing::info;

use crate::mm;

pub struct Scheduler {
    tss: Box<mm::sm::TaskStateSegment>,

    stack_ptr: *mut u8,
    work_queue: VecDeque<Thread>,
    work: Option<Thread>,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self {
            tss: Default::default(),
            stack_ptr: ptr::null_mut(),
            work_queue: Default::default(),
            work: Default::default(),
        }
    }
}

impl Scheduler {
    pub unsafe fn get() -> &'static mut Self {
        let ctx: *mut Self;
        arch::asm!("mov {ctx}, gs:0", ctx = out(reg) ctx);
        &mut *ctx
    }

    pub fn run(&mut self) -> ! {
        while let Some(thread) = self.work_queue.pop_front() {
            self.work = Some(thread);
            unsafe {
                context_swap(
                    self.work.as_ref().unwrap_unchecked().stack_ptr,
                    &mut self.stack_ptr,
                );
            }
        }
        panic!("Reached end of scheduler");
    }

    pub fn enter(&mut self, interrupt: bool) {
        if interrupt {
            if let Some(work) = self.work.take() {
                self.work_queue.push_back(work);
                unsafe {
                    arch::asm!("sti");
                    context_swap(
                        self.stack_ptr,
                        &mut self.work_queue.back_mut().unwrap_unchecked().stack_ptr,
                    );
                }
            } else {
                panic!("Entered while not in a thread")
            }
        } else {
            if let Some(work) = self.work.as_ref() {
                unsafe {
                    arch::asm!("sti");
                }
                (work.method)(work.method_arg);
                self.work = None;
                unsafe {
                    context_swap(self.stack_ptr, &mut ptr::null_mut());
                }
            } else {
                panic!("Entered while not in a thread")
            }
        }
    }

    pub fn spawn(&mut self, method: fn(usize) -> (), method_arg: usize) {
        self.work_queue
            .push_back(Thread::new(entry_point_thread, method, method_arg));
    }
}

pub struct Thread {
    stack: Box<[u8]>,
    stack_ptr: *mut u8,

    method: fn(usize) -> (),
    method_arg: usize,
}

impl Thread {
    fn new(entry_point: fn() -> !, method: fn(usize) -> (), method_arg: usize) -> Self {
        let mut stack = unsafe { Box::<[u8; 16 * 1024]>::new_uninit().assume_init() };
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
            stack,
            stack_ptr,
            method,
            method_arg,
        }
    }
}

pub fn run() -> ! {
    fn dummy(_: usize) {}

    unsafe {
        context_swap(
            Thread::new(entry_point_scheduler, dummy, 0).stack_ptr,
            &mut ptr::null_mut(),
        );
        hint::unreachable_unchecked()
    }
}

fn entry_point_scheduler() -> ! {
    let mut scheduler: Box<Scheduler> = Box::default();
    scheduler.tss.set();
    mm::sm::GS::set(ptr::addr_of!(scheduler) as usize, size_of::<Scheduler>());

    scheduler.spawn(test_method, 0);
    scheduler.spawn(test_method, 1);
    scheduler.spawn(test_method, 2);
    scheduler.spawn(test_method, 3);
    scheduler.run()
}

fn entry_point_thread() -> ! {
    unsafe { Scheduler::get() }.enter(false);
    unsafe { hint::unreachable_unchecked() }
}

fn test_method(arg: usize) {
    loop {
        info!("Called {}", arg);
        unsafe { arch::asm!("hlt") };
    }
}

extern "C" {
    fn context_swap(load: *mut u8, save: &mut *mut u8);
}
