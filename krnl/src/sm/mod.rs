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

mod ctx;

use core::{arch, cell, mem, ptr};

use alloc::{boxed::Box, collections::vec_deque::VecDeque};

pub static SCHED: cell::SyncUnsafeCell<Option<Scheduler>> = cell::SyncUnsafeCell::new(None);

pub struct Scheduler {
    queue: VecDeque<Schedulable>,

    prev: Schedulable,
    next: Option<Schedulable>,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self {
            queue: Default::default(),
            prev: Schedulable {
                stack: unsafe {
                    mem::transmute(ptr::slice_from_raw_parts(
                        ptr::NonNull::<[usize; 0]>::dangling().as_ptr() as *const _,
                        0,
                    ))
                },
                context: ctx::Context::zeroed(),
            },
            next: None,
        }
    }
}

impl Scheduler {
    pub fn run(&mut self) {
        unsafe {
            arch::asm!("sti");
        }

        while let Some(next) = self.queue.pop_front() {
            self.next = Some(next);
            ctx::Context::swap(&mut self.prev.context, &self.next.as_ref().unwrap().context);
        }
    }

    pub fn spawn(&mut self, entry_point: fn() -> !) {
        self.queue.push_back(Schedulable::new(entry_point));
    }

    pub fn r#yield(&mut self) {
        if let Some(next) = self.next.take() {
            self.queue.push_back(next);
            ctx::Context::swap(
                &mut self.queue.back_mut().unwrap().context,
                &self.prev.context,
            );
        }
    }
}

struct Schedulable {
    stack: Box<[u8]>,
    context: ctx::Context,
}

impl Schedulable {
    pub fn new(entry_point: fn() -> !) -> Self {
        let mut stack = unsafe { Box::<[u8; 16 * 1024]>::new_uninit().assume_init() };
        let context =
            unsafe { ctx::Context::new(entry_point, stack.as_mut_ptr() as *mut (), 16 * 1024) };
        Self { stack, context }
    }
}

pub fn init_and_run() {
    let sched = unsafe { &mut *SCHED.get() };
    *sched = Some(Scheduler::default());

    sched.as_mut().unwrap().run();
}
